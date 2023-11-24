use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NominatimPlace {
    pub osm_type: String,
    pub place_id: usize,
    pub display_name: String,
    pub address: HashMap<String, String>,
    pub extratags: ExtraTags,
}

#[derive(Serialize, Deserialize)]
pub struct ExtraTags {
    pub surface: Option<String>,
}
