use std::collections::HashMap;

use geo_types::{LineString, Point};
use geojson::GeoJson;
use google_maps::AddressComponent;
use serde::{Deserialize, Serialize};
use sqlx::types::{BigDecimal, Json};

#[derive(Serialize, Deserialize)]
pub struct Ride {
    pub id: Option<i64>,
    pub name: String,
    pub geo_json: Json<GeoJson>,
    //Total distance in metres
    pub total_distance: BigDecimal,
    pub surface_composition: Json<HashMap<String, BigDecimal>>,
    pub ways: Json<HashMap<String, BigDecimal>>,
    //Ride time in seconds
    pub ride_time: i64,
    pub start_address: Json<Vec<AddressComponent>>,
    pub end_address: Json<Vec<AddressComponent>>,
}

#[derive(Serialize, Deserialize)]
pub struct ListRide {
    pub id: i64,
    pub name: String,
    pub total_distance: BigDecimal,
    pub surface_composition: Json<HashMap<String, BigDecimal>>,
    pub ride_time: i64,
    pub start_address: Json<Vec<AddressComponent>>,
    pub end_address: Json<Vec<AddressComponent>>,
    pub start_point: Option<Json<Point>>,
    pub end_point: Option<Json<Point>>,
}

#[derive(Serialize, Deserialize)]
pub struct Way {
    pub osm_id: i64,
    pub name: String,
    pub surface_composition: String,
}

#[derive(Serialize, Deserialize)]
pub struct RideWay {
    pub ride_id: i64,
    pub way_osm_id: i64,
    pub distance_on_ride: BigDecimal,
}
