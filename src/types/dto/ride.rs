use std::collections::HashMap;

use geojson::GeoJson;
use serde::{Deserialize, Serialize};
use sqlx::types::{BigDecimal, Json};

use crate::types::model::ride::RideWay;

#[derive(Serialize, Deserialize)]
pub struct ListRide {
    pub id: i64,
    pub name: String,
    pub total_distance: BigDecimal,
    pub start_address: Json<HashMap<String, String>>,
    pub end_address: Json<HashMap<String, String>>,
    pub time_from_origin_to_start: i64,
    pub time_from_end_to_origin: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Ride {
    pub id: i64,
    pub name: String,
    pub geo_json: Json<GeoJson>,
    pub ways: Json<Vec<RideWay>>,
    pub total_distance: BigDecimal,
    pub start_address: Json<HashMap<String, String>>,
    pub end_address: Json<HashMap<String, String>>,
    pub time_from_origin_to_start: i64,
    pub time_from_end_to_origin: i64,
}
