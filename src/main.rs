#![feature(async_closure)]
#![feature(iter_map_windows)]
#![feature(iter_intersperse)]

mod clients;
mod import;
mod net;
mod ride;
mod ride_geo;
mod ride_processing;
mod types;

use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, Query},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use clients::{get_db_pool, DB_POOL, GMAPS, REQWEST};
use color_eyre::eyre::eyre;
use futures::stream::TryStreamExt;
use futures::{stream, StreamExt};
use geojson::FeatureCollection;
use google_maps::GoogleMapsClient;
use net::response::{ResponseError, Result};
use ride::create_ride;
use ride_processing::{aggregate_surface, nominatim_get_place, process_ride};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tracing::{info, instrument};
use types::dto::{self, geom::PartialLatLng};
use types::model;

use crate::{clients::NOMINATIM_URL, import::gpx::AsRideFeatureCollection};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // initialize tracing
    tracing_subscriber::fmt::init();

    init_db().await?;
    init_google_maps()?;
    NOMINATIM_URL
        .set(std::env::var("EXPEDITION_NOMINATIM_URL")?)
        .unwrap();
    init_reqwest_client()?;

    // build our application with a route
    let app = Router::new()
        .route("/gpx", post(import_gpx))
        .route("/rides", get(list_rides))
        .route("/rides/:id", get(get_ride_by_id))
        .route("/rides/:id", delete(delete_ride_by_id))
        .layer(CorsLayer::permissive());

    info!("Running on port 3000");

    // run our app with hyper, listening globally on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn init_db() -> color_eyre::Result<()> {
    let db_uri = std::env::var("DATABASE_URL")?;
    info!("Connecting to db");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_uri)
        .await?;
    DB_POOL.set(db_pool).unwrap();
    info!("Connected");
    Ok(())
}

fn init_google_maps() -> color_eyre::Result<()> {
    let google_api_key = std::env::var("EXPEDITION_GOOGLE_API_KEY")?;
    let google_maps_client = GoogleMapsClient::new(&google_api_key);
    GMAPS.set(google_maps_client).unwrap();
    Ok(())
}

fn init_reqwest_client() -> color_eyre::Result<()> {
    let client = reqwest::Client::new();
    REQWEST.set(client).unwrap();
    Ok(())
}

async fn list_rides(Query(origin): Query<PartialLatLng>) -> Result<Json<Vec<dto::ride::ListRide>>> {
    let rides = sqlx::query_as!(
        model::ride::QueryRide,
        r#"select 
        id,
        name,
        total_distance,
        null as "ways: _",
        null as "geo_json: _",
        jsonb_path_query(geo_json, '$[*].features ? (@.id == "start").geometry.coordinates') as "start_point: _",
        jsonb_path_query(geo_json, '$[*].features ? (@.id == "end").geometry.coordinates') as "end_point: _"
        from rides"#
    )
    .fetch_all(get_db_pool()?)
    .await?;

    let rides = stream::iter(rides.into_iter())
        .map(|ride| {
            let origin = origin.clone();
            async move {
                let processed_ride = process_ride(ride, origin.into()).await?;

                Result::<dto::ride::ListRide>::Ok(dto::ride::ListRide {
                    id: processed_ride.id,
                    name: processed_ride.name,
                    total_distance: processed_ride.total_distance,
                    start_address: processed_ride.start_address.into(),
                    end_address: processed_ride.end_address.into(),
                    time_from_origin_to_start: processed_ride.time_from_origin_to_start,
                    time_from_end_to_origin: processed_ride.time_from_end_to_origin,
                })
            }
        })
        .buffered(10)
        .try_collect::<Vec<dto::ride::ListRide>>()
        .await?;

    Ok(Json(rides))
}

async fn get_ride_by_id(
    Path(ride_id): Path<i64>,
    Query(origin): Query<PartialLatLng>,
) -> Result<Json<dto::ride::Ride>> {
    // jsonb_path_query_array(ways, '$[0 to 9]') as "ways: sqlx_json<Vec<model::ride::RideWay>>",
    let option_ride = sqlx::query_as!(
        model::ride::QueryRide,
        r#"select
        id,
        name,
        total_distance,
        geo_json as "geo_json: _",
        ways as "ways: _",
        jsonb_path_query(geo_json, '$[*].features ? (@.id == "start").geometry.coordinates') as "start_point: _",
        jsonb_path_query(geo_json, '$[*].features ? (@.id == "end").geometry.coordinates') as "end_point: _"
        from rides
        where id = $1"#,
        ride_id.into()
    )
    .fetch_optional(get_db_pool()?)
    .await?;
    let query_ride = Arc::new(option_ride.ok_or(ResponseError::not_found("No ride with this id"))?);
    let ways: Vec<dto::ride::RideWay> = stream::iter(
        query_ride
            .clone()
            .ways
            .clone()
            .ok_or(eyre!("No ways!"))?
            .0
            .into_iter(),
    )
    .map(|way| async move {
        let mut place = nominatim_get_place("W", way.osm_id).await?;
        place.extratags.surface = place
            .extratags
            .surface
            .map(|s| aggregate_surface(&s).to_string());
        Result::<dto::ride::RideWay>::Ok(dto::ride::RideWay {
            distance: way.distance,
            points: way.points,
            place,
        })
    })
    .buffered(10)
    .try_collect()
    .await?;
    let processed_ride = process_ride(
        Arc::try_unwrap(query_ride).expect("Couldnt unwrap queryride"),
        origin.into(),
    )
    .await?;
    let ride = dto::ride::Ride {
        id: processed_ride.id,
        name: processed_ride.name,
        total_distance: processed_ride.total_distance,
        geo_json: processed_ride.geo_json.ok_or(eyre!("No geo_json!"))?,
        ways: ways.into(),
        start_address: processed_ride.start_address.into(),
        end_address: processed_ride.end_address.into(),
        time_from_origin_to_start: processed_ride.time_from_origin_to_start,
        time_from_end_to_origin: processed_ride.time_from_end_to_origin,
    };
    Ok(Json(ride))
}

async fn delete_ride_by_id(Path(ride_id): Path<i64>) -> Result<()> {
    let result = sqlx::query!(
        r#"delete from rides 
        where id = $1"#,
        ride_id
    )
    .execute(get_db_pool()?)
    .await?;
    if result.rows_affected() == 0 {
        Err(ResponseError::not_found("no ride with this id"))?;
    }
    Ok(())
}

#[instrument(skip(multipart))]
#[axum::debug_handler]
async fn import_gpx(mut multipart: Multipart) -> Result<()> {
    let mut ride_name_opt: Option<String> = None;
    let mut geo_feature_collection_opt: Option<FeatureCollection> = None;
    // let mut ride_name_opt: Option<String> = None;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or(ResponseError::internal_server_error(
            "No name on form field",
        ))?;
        match name {
            "ride_name" => ride_name_opt = Some(field.text().await?),
            "gpx" => {
                let gpx_obj = gpx::read(field.text().await?.as_bytes())?;
                geo_feature_collection_opt = Some(gpx_obj.as_ride_feature_collection()?);
            }
            _ => continue,
        }
    }
    let ride_name = ride_name_opt.ok_or(ResponseError::with_status(
        StatusCode::BAD_REQUEST,
        "ride_name not provided",
    ))?;
    let geo_feature_collection = geo_feature_collection_opt.ok_or(ResponseError::with_status(
        StatusCode::BAD_REQUEST,
        "gpx not provided",
    ))?;
    let ride = create_ride(
        geo_feature_collection
            .features
            .iter()
            .map(|feature| -> Result<&str> {
                match feature.property("name") {
                    Some(name) => name
                        .as_str()
                        .ok_or(ResponseError::bad_request("name is not a string")),
                    None => Err(ResponseError::bad_request("Unnamed feature")),
                }
            })
            .collect::<Result<Vec<&str>>>()?
            .into_iter()
            .intersperse(" / ")
            .collect(),
        geo_feature_collection,
    )
    .await?;
    sqlx::query!(
        r#"insert into rides (name, geo_json, total_distance, ways)
        values ($1, $2, $3, $4)"#,
        ride.name,
        ride.geo_json as _,
        ride.total_distance,
        ride.ways as _,
    )
    .execute(get_db_pool()?)
    .await?;
    Ok(())
}
