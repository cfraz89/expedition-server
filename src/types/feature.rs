use color_eyre::eyre;
use color_eyre::eyre::eyre;
use geojson::JsonObject;
use serde::{Deserialize, Serialize};

/// Properties that are attached to a geojson feature
#[derive(Serialize, Deserialize)]
pub struct FeatureProperties {
    pub distance: f64,
    pub name: Option<String>,
}

/// For converting FeatureProperties to geojson properties
impl TryInto<JsonObject> for FeatureProperties {
    type Error = eyre::Error;

    fn try_into(self) -> Result<JsonObject, Self::Error> {
        let value = serde_json::to_value(self)?;
        let properties = value
            .as_object()
            .ok_or(eyre!("Couldn't create object for properties"))?;
        Ok(properties.to_owned())
    }
}
