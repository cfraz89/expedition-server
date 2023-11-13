use geo_types::Point;
use geojson::GeoJson;
use google_maps::AddressComponent;
use serde::{Deserialize, Serialize};
use sqlx::types::{BigDecimal, Json};

#[derive(Serialize, Deserialize)]
pub struct Ride {
    pub id: Option<i32>,
    pub name: String,
    pub geo_json: Json<GeoJson>,
    pub total_distance: BigDecimal,
    pub start_address: Json<Vec<AddressComponent>>,
    pub end_address: Json<Vec<AddressComponent>>,
}

#[derive(Serialize, Deserialize)]
pub struct ListRide {
    pub id: i32,
    pub name: String,
    pub total_distance: BigDecimal,
    pub start_address: Json<Vec<AddressComponent>>,
    pub end_address: Json<Vec<AddressComponent>>,
    pub start_point: Option<Json<Point>>,
}
