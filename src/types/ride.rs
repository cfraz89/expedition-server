use geojson::GeoJson;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Serialize, Deserialize)]
pub struct Ride {
    pub id: Option<Thing>,
    pub name: String,
    pub geo_json: GeoJson,
    pub total_distance: f64,
}
