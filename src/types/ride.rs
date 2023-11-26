use std::collections::HashMap;

use geo_types::Point;
use geojson::GeoJson;
use serde::{Deserialize, Serialize};
use sqlx::types::{BigDecimal, Json};

#[derive(Serialize, Deserialize)]
pub struct Ride {
    pub id: Option<i64>,
    pub name: String,
    pub geo_json: Json<GeoJson>,
    //Total distance in metres
    pub total_distance: BigDecimal,
    pub ways: Json<Vec<Way>>,
    pub start_address: Json<HashMap<String, String>>,
    pub end_address: Json<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct ListRide {
    pub id: i64,
    pub name: String,
    pub total_distance: BigDecimal,
    pub ways: Json<Vec<Way>>,
    pub start_address: Json<HashMap<String, String>>,
    pub end_address: Json<HashMap<String, String>>,
    pub start_point: Option<Json<Point>>,
    pub end_point: Option<Json<Point>>,
}

#[derive(Serialize, Deserialize)]
pub struct Way {
    pub distance: f64,
    pub points: Vec<WayPoint>,
    pub address: HashMap<String, String>,
    pub surface: Option<String>,
}
#[derive(Serialize, Deserialize)]
pub struct WayPoint {
    pub seq: usize,
    pub point: Point,
}
