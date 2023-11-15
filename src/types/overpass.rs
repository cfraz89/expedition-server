use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OverpassResponse {
    pub elements: Vec<OverpassElement>,
}

#[derive(Serialize, Deserialize)]
pub struct OverpassElement {
    #[serde(rename = "type")]
    pub element_type: String,
    pub id: i64,
    pub nodes: Option<Vec<i64>>,
    pub geometry: Option<Vec<OverpassCoord>>,
    pub tags: OverpassTags,
}

#[derive(Serialize, Deserialize)]
pub struct OverpassCoord {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Serialize, Deserialize)]
pub struct OverpassTags {
    pub highway: Option<String>,
    pub name: Option<String>,
    pub surface: Option<String>,
    pub trail_visibility: Option<String>,
}
