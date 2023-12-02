use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NominatimPlace {
    pub osm_type: String,
    pub osm_id: u64,
    pub display_name: String,
    pub address: HashMap<String, String>,
    pub extratags: ExtraTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtraTags {
    #[serde(default)]
    pub surface: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NominatimDetailsPlace {
    pub osm_type: String,
    pub osm_id: u64,
    pub localname: String,
    pub extratags: ExtraTags,
}
