mod net;

use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use geo::algorithm::vincenty_distance::VincentyDistance;
use geo_types::Point;
use geojson::{GeoJson, JsonObject};
use net::response::{ResponseError, Result};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Thing,
    Surreal,
};
use tower_http::cors::CorsLayer;
use tracing::{info, instrument};

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

#[derive(Serialize, Deserialize)]
struct Ride {
    id: Option<Thing>,
    name: String,
    geo_json: GeoJson,
    total_distance: f64,
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
    let mut total_distance = 0.0;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or(ResponseError::internal_server_error(
            "No name on form field",
        ))?;
        match name {
            "ride_name" => ride_name_opt = Some(field.text().await?),
            "gpx" => {
                let text = field.text().await?;
                let gpx_data = gpx::read(text.as_bytes())?;
                info!("number of tracks in gpx: {}", gpx_data.tracks.len());
                let geo_json: GeoJson = gpx_data
                    .tracks
                    .into_iter()
                    .map(|track: gpx::Track| {
                        // println!("{:?}", track);
                        let mls = track.multilinestring();
                        let geo_meta = get_geo_meta(&mls);
                        let bounding_box = &geo_meta.bounding_box;
                        let center = bounding_box.as_ref().map(get_center);
                        let mut properties = JsonObject::new();
                        properties.insert(String::from("center"), center.to_owned().into());
                        properties.insert(String::from("distance"), geo_meta.distance.into());
                        total_distance += geo_meta.distance;
                        return geojson::Feature {
                            bbox: bounding_box.to_owned(),
                            geometry: Some(geojson::Geometry {
                                bbox: bounding_box.to_owned(),
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
            total_distance,
        })
        .await?;

    Ok(())
}

struct GeoMeta {
    bounding_box: Option<Vec<f64>>,
    distance: f64,
}

fn get_geo_meta(multi_line_string: &geo_types::MultiLineString) -> GeoMeta {
    let mut min_x: Option<f64> = None;
    let mut min_y: Option<f64> = None;
    let mut max_x: Option<f64> = None;
    let mut max_y: Option<f64> = None;
    let mut distance = 0.0;
    for line_string in multi_line_string {
        let mut previous_point: Option<Point> = None;
        for point in line_string.points() {
            let x = point.x();
            let y = point.y();
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
            if let Some(pc) = previous_point {
                if let Ok(d) = point.vincenty_distance(&pc) {
                    distance += d;
                }
            }
            previous_point = Some(point);
        }
    }
    let bounding_box: Option<Vec<f64>> = vec![min_x, min_y, max_x, max_y].into_iter().collect();
    return GeoMeta {
        bounding_box,
        distance,
    };
}

fn get_center(bounding_box: &Vec<f64>) -> Vec<f64> {
    if let [min_x, min_y, max_x, max_y] = bounding_box[..] {
        return vec![(max_x - min_x) / 2.0, (max_y - min_y) / 2.0];
    };
    return vec![0.0, 0.0];
}
