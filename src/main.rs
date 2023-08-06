mod net;

use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use geojson::GeoJson;
use gpx::{Gpx, Waypoint};
use net::response::{ResponseError, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};

static DB: OnceLock<Surreal<Client>> = OnceLock::new();

fn get_db() -> Result<&'static Surreal<Client>> {
    DB.get()
        .ok_or(ResponseError::internal_server_error("Failed to get db"))
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // initialize tracing
    tracing_subscriber::fmt::init();
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;
    db.use_ns("ausadv").use_db("ausadv").await?;
    DB.set(db).unwrap();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/rides", get(get_rides))
        .route("/gpx", post(import_gpx));

    // run our app with hyper, listening globally on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Ride {
    name: String,
    geo_json: GeoJson,
}

async fn get_rides() -> Result<Json<Vec<serde_json::Value>>> {
    let mut rides = get_db()?
        .query("select meta::id(id) as id, name from rides")
        .await?;
    let ride_names: Vec<serde_json::Value> = rides.take(0)?;
    Ok(Json(ride_names))
}

async fn import_gpx(mut multipart: Multipart) -> Result<impl IntoResponse> {
    let mut geo_json_opt: Option<GeoJson> = None;
    let mut ride_name_opt: Option<String> = None;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or(ResponseError::internal_server_error(
            "No name on form field",
        ))?;
        match name {
            "ride_name" => ride_name_opt = Some(field.text().await?),
            "gpx" => {
                let text = field.text().await?;
                let gpx_data = gpx::read(text.as_bytes())?;
                let gj: geojson::FeatureCollection = gpx_data
                    .tracks
                    .into_iter()
                    .map(|track: gpx::Track| {
                        // println!("{:?}", track);
                        return geojson::Feature {
                            geometry: Some(geojson::Geometry::new(geojson::Value::Polygon(
                                track
                                    .segments
                                    .into_iter()
                                    .map(|segment| {
                                        segment
                                            .points
                                            .into_iter()
                                            .map(|point: Waypoint| {
                                                vec![point.point().x(), point.point().y()]
                                            })
                                            .collect::<Vec<Vec<f64>>>()
                                    })
                                    .collect::<Vec<Vec<Vec<f64>>>>(),
                            ))),
                            ..Default::default()
                        };
                    })
                    .collect();
                geo_json_opt = Some(GeoJson::FeatureCollection(gj));
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
    let _ride: Ride = get_db()?
        .create("rides")
        .content(Ride {
            name: ride_name,
            geo_json,
        })
        .await?;

    Ok("ok")
}
