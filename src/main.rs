mod net;

use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use geo_types::Coord;
use geojson::{GeoJson, JsonObject};
use gpx::{Gpx, TrackSegment, Waypoint};
use net::response::{ResponseError, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::{Id, Thing},
    Surreal,
};
use tower_http::cors::CorsLayer;

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
        .route("/gpx", post(import_gpx))
        .route("/rides", get(get_rides))
        .route("/rides/:id", get(get_ride_by_id))
        .layer(CorsLayer::permissive());

    // run our app with hyper, listening globally on port 3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Ride {
    id: Option<Thing>,
    name: String,
    geo_json: GeoJson,
}

// struct RideGeo {
//     geo_json: GeoJson,
//     min_x: Option<f64>,
//     min_y: Option<f64>,
//     max_x: Option<f64>,
//     max_y: Option<f64>,
// }

async fn get_rides() -> Result<Json<Vec<serde_json::Value>>> {
    let mut rides = get_db()?
        .query("select meta::id(id) as id, name from rides")
        .await?;
    let ride_names: Vec<serde_json::Value> = rides.take(0)?;
    Ok(Json(ride_names))
}

async fn get_ride_by_id(Path(ride_id): Path<String>) -> Result<Json<Ride>> {
    let option_ride: Option<Ride> = get_db()?.select(("rides", ride_id)).await?;
    let ride = option_ride.ok_or(ResponseError::not_found("No ride with this id"))?;
    Ok(Json(ride))
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
                let geo_json: geojson::GeoJson = gpx_data
                    .tracks
                    .into_iter()
                    .map(|track: gpx::Track| {
                        // println!("{:?}", track);
                        let mls = track.multilinestring();
                        let bbox = get_bounding_box(&mls);
                        let center = bbox.clone().map(get_center);
                        let mut properties = JsonObject::new();
                        properties.insert(String::from("center"), center.into());
                        return geojson::Feature {
                            bbox: bbox.clone(),
                            geometry: Some(geojson::Geometry {
                                bbox: bbox,
                                value: geojson::Value::from(&mls),
                                foreign_members: None,
                            }),
                            properties: Some(properties),
                            ..Default::default()
                        };
                    })
                    .collect::<geojson::FeatureCollection>()
                    .into();
                geo_json_opt = Some(geo_json)
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
            id: None,
            name: ride_name,
            geo_json,
        })
        .await?;

    Ok("ok")
}

fn get_bounding_box(multi_line_string: &geo_types::MultiLineString) -> Option<Vec<f64>> {
    let mut min_x: Option<&f64> = None;
    let mut min_y: Option<&f64> = None;
    let mut max_x: Option<&f64> = None;
    let mut max_y: Option<&f64> = None;
    for line_string in multi_line_string {
        for Coord { x, y } in line_string.coords() {
            min_x = match min_x {
                None => Some(x),
                Some(mx) if x < mx => Some(x),
                Some(..) => min_x,
            };
            min_y = match min_y {
                None => Some(y),
                Some(my) if y < my => Some(y),
                Some(..) => min_y,
            };
            max_x = match max_x {
                None => Some(x),
                Some(mx) if x > mx => Some(x),
                Some(..) => max_x,
            };
            max_y = match max_y {
                None => Some(y),
                Some(my) if y > my => Some(y),
                Some(..) => max_y,
            };
        }
    }
    match (min_x, min_y, max_x, max_y) {
        (Some(min_x), Some(min_y), Some(max_x), Some(max_y)) => {
            Some(vec![*min_x, *min_y, *max_x, *max_y])
        }
        _ => None,
    }
}

fn get_center(bounding_box: Vec<f64>) -> Vec<f64> {
    if let [min_x, min_y, max_x, max_y] = bounding_box[..] {
        return vec![(max_x - min_x) / 2.0, (max_y - min_y) / 2.0];
    };
    return vec![0.0, 0.0];
}
