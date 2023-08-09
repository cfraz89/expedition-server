mod import;
mod net;
mod ride_geo;
mod types;

use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use geojson::GeoJson;
use google_maps::prelude::*;
use net::response::{ResponseError, Result};
use ride_geo::{Distance, IntoRideGeoJson};
use std::sync::OnceLock;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use tower_http::cors::CorsLayer;
use tracing::instrument;
use types::ride::Ride;

static DB: OnceLock<Surreal<Client>> = OnceLock::new();
static GMAPS: OnceLock<GoogleMapsClient> = OnceLock::new();

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

    // run our app with hyper, listening globally on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn init_db() -> color_eyre::Result<()> {
    let db_uri = std::env::var("EXPEDITION_SURREAL_URI")?;
    let db_user = std::env::var("EXPEDITION_SURREAL_USER")?;
    let db_pass = std::env::var("EXPEDITION_SURREAL_PASS")?;
    let db = Surreal::new::<Ws>(db_uri).await?;
    db.signin(Root {
        username: db_user.as_str(),
        password: db_pass.as_str(),
    })
    .await?;
    db.use_ns("expedition").use_db("expedition").await?;
    DB.set(db).unwrap();
    Ok(())
}

fn init_google_maps() -> color_eyre::Result<()> {
    let google_api_key = std::env::var("EXPEDITION_GOOGLE_API_KEY")?;
    let google_maps_client = GoogleMapsClient::new(&google_api_key);
    GMAPS.set(google_maps_client).unwrap();
    Ok(())
}

fn get_db() -> Result<&'static Surreal<Client>> {
    DB.get()
        .ok_or(ResponseError::internal_server_error("Failed to get db"))
}

async fn get_rides() -> Result<Json<Vec<serde_json::Value>>> {
    let mut rides = get_db()?
        .query("select meta::id(id) as id, name, total_distance from rides")
        .await?;
    let ride_names: Vec<serde_json::Value> = rides.take(0)?;
    Ok(Json(ride_names))
}

async fn get_ride_by_id(Path(ride_id): Path<String>) -> Result<Json<Ride>> {
    let option_ride: Option<Ride> = get_db()?.select(("rides", ride_id)).await?;
    let ride = option_ride.ok_or(ResponseError::not_found("No ride with this id"))?;
    Ok(Json(ride))
}

async fn delete_ride_by_id(Path(ride_id): Path<String>) -> Result<()> {
    let ride: Option<Ride> = get_db()?.delete(("rides", ride_id)).await?;
    ride.map(|_| ())
        .ok_or(ResponseError::not_found("no ride with this id"))
}

#[instrument(skip(multipart))]
async fn import_gpx(mut multipart: Multipart) -> Result<()> {
    let mut geo_json_opt: Option<GeoJson> = None;
    let mut ride_name_opt: Option<String> = None;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or(ResponseError::internal_server_error(
            "No name on form field",
        ))?;
        match name {
            "ride_name" => ride_name_opt = Some(field.text().await?),
            "gpx" => {
                let gpx_obj = gpx::read(field.text().await?.as_bytes())?;
                geo_json_opt = Some(gpx_obj.into_ride_geo_json()?);
            }
            _ => continue,
        }
    }
    let ride_name = ride_name_opt.ok_or(ResponseError::with_status(
        StatusCode::BAD_REQUEST,
        "ride_name not provided",
    ))?;
    let geo_json = geo_json_opt.ok_or(ResponseError::with_status(
        StatusCode::BAD_REQUEST,
        "gpx not provided",
    ))?;
    let total_distance = geo_json.distance();
    let _ride: Ride = get_db()?
        .create("rides")
        .content(Ride {
            id: None,
            name: ride_name,
            geo_json,
            total_distance,
        })
        .await?;

    Ok(())
}
