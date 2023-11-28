use std::collections::HashMap;

use geo_types::Point;
use geojson::GeoJson;
use serde::{Deserialize, Serialize};
use sqlx::types::{BigDecimal, Json};

//Whats actually stored in the db
#[derive(Serialize, Deserialize)]
pub struct Ride {
    pub id: Option<i64>,
    pub name: String,
    pub geo_json: Json<GeoJson>,
    //Total distance in metres
    pub total_distance: BigDecimal,
    pub ways: Json<Vec<RideWay>>,
}

//Used when retrieving from db
#[derive(Serialize, Deserialize)]
pub struct QueryRide {
    pub id: i64,
    pub name: String,
    pub total_distance: BigDecimal,
    pub geo_json: Option<Json<GeoJson>>,
    pub ways: Option<Json<Vec<RideWay>>>,
    pub start_point: Option<Json<Point>>,
    pub end_point: Option<Json<Point>>,
}

//Used when retrieving from db
pub struct ProcessedRide {
    pub id: i64,
    pub name: String,
    pub total_distance: BigDecimal,
    pub geo_json: Option<Json<GeoJson>>,
    pub ways: Option<Json<Vec<RideWay>>>,
    pub start_point: Point,
    pub end_point: Point,
    pub start_address: HashMap<String, String>,
    pub end_address: HashMap<String, String>,
    pub time_from_origin_to_start: i64,
    pub time_from_end_to_origin: i64,
}

#[derive(Serialize, Deserialize)]
pub struct RideWay {
    pub place_id: u64,
    pub distance: f64,
    pub points: Vec<WayPoint>,
}

#[derive(Serialize, Deserialize)]
pub struct WayPoint {
    pub seq: usize,
    pub point: Point,
}
