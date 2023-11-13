mod clients;
mod geocode;
mod import;
mod net;
mod ride;
mod ride_geo;
mod types;

use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use clients::{get_db_pool, DB_POOL, GMAPS};
use geojson::FeatureCollection;
use google_maps::GoogleMapsClient;
use net::response::{ResponseError, Result};
use ride::create_ride;
use ride_geo::IntoRideFeatureCollection;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tracing::{info, instrument};
use types::ride::{ListRide, Ride};
use sqlx::types::Json as sqlx_json;
use google_maps::AddressComponent

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // initialize tracing
    tracing_subscriber::fmt::init();

    init_db().await?;
    init_google_maps()?;

    // build our application with a route
    let app = Router::new()
        .route("/gpx", post(import_gpx))
        .route("/rides", get(get_rides))
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
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_uri)
        .await?;
    DB_POOL.set(db_pool).unwrap();
    Ok(())
}

fn init_google_maps() -> color_eyre::Result<()> {
    let google_api_key = std::env::var("EXPEDITION_GOOGLE_API_KEY")?;
    let google_maps_client = GoogleMapsClient::new(&google_api_key);
    GMAPS.set(google_maps_client).unwrap();
    Ok(())
}

async fn get_rides() -> Result<Json<Vec<ListRide>>> {
    let rides = sqlx::query_as!(
        ListRide,
        r#"select 
        id,
        name,
        total_distance,
        start_address as "start_address: sqlx_json<Vec<AddressComponent>>",
        end_address as "end_address: sqlx_json<Vec<AddressComponent>>", 
        jsonb_path_query(geo_json, '$[*].features ? (@.id == "start").geometry.coordinates') as "start_point: sqlx_json<geo_types::Point>"
        from rides"#
    )
    .fetch_all(get_db_pool()?)
    .await?;
    Ok(Json(rides))
}

async fn get_ride_by_id(Path(ride_id): Path<i32>) -> Result<Json<Ride>> {
    let option_ride = sqlx::query_as!(
        Ride,
        r#"select
    id,
    name,
    geo_json as "geo_json: sqlx_json<geojson::GeoJson>",
    total_distance,
    start_address as "start_address: sqlx_json<Vec<AddressComponent>>",
    end_address as "end_address: sqlx_json<Vec<AddressComponent>>"
 from rides 
        where id = $1"#,
        ride_id.into()
    )
    .fetch_optional(get_db_pool()?)
    .await?;
    let ride = option_ride.ok_or(ResponseError::not_found("No ride with this id"))?;
    Ok(Json(ride))
}

async fn delete_ride_by_id(Path(ride_id): Path<i32>) -> Result<()> {
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
    let mut geo_feature_collection_opt: Option<FeatureCollection> = None;
    let mut ride_name_opt: Option<String> = None;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or(ResponseError::internal_server_error(
            "No name on form field",
        ))?;
        match name {
            "ride_name" => ride_name_opt = Some(field.text().await?),
            "gpx" => {
                let gpx_obj = gpx::read(field.text().await?.as_bytes())?;
                geo_feature_collection_opt = Some(gpx_obj.into_ride_feature_collection()?);
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
    let ride = create_ride(ride_name, geo_feature_collection).await?;
    sqlx::query!(
        r#"insert into rides (name, geo_json, total_distance, start_address, end_address)
        values ($1, $2, $3, $4, $5)"#,
        ride.name,
        ride.geo_json as _,
        ride.total_distance,
        ride.start_address as _,
        ride.end_address as _,
    )
    .execute(get_db_pool()?)
    .await?;
    Ok(())
}
