use geo_types::Point;
use geojson::GeoJson;
use google_maps::AddressComponent;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize)]
pub struct Ride {
    pub id: Option<Thing>,
    pub name: String,
    pub geo_json: GeoJson,
    pub total_distance: f64,
    pub start_address: Vec<AddressComponent>,
    pub end_address: Vec<AddressComponent>,
}

#[derive(Serialize, Deserialize)]
pub struct ListRide {
    pub id: String,
    pub name: String,
    pub total_distance: f64,
    pub start_address: Vec<AddressComponent>,
    pub end_address: Vec<AddressComponent>,
    pub start_point: Point,
}
