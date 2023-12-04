use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NominatimPlace {
    pub osm_type: String,
    pub osm_id: u64,
    pub display_name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    pub address: Address,
    pub extratags: ExtraTags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    pub road: String,
    #[serde(default)]
    pub tourism: Option<String>,
    #[serde(default)]
    pub amenity: Option<String>,
    #[serde(default)]
    pub suburb: Option<String>,
    #[serde(default)]
    pub hamlet: Option<String>,
    #[serde(default)]
    pub town: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub municipality: Option<String>,
    pub state: String,
    #[serde(rename = "ISO3166-2-lvl4")]
    pub iso3166_2_lvl4: String,
    #[serde(default)]
    pub postcode: Option<String>,
    pub country: String,
    pub country_code: String,
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
