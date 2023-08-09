use color_eyre::Result;
use geo::VincentyDistance;
use geo_types::Point;
use geojson::{Feature, GeoJson};
use tracing::{info, instrument};

use crate::lib::ride::IntoRideFeature;

#[instrument]
pub fn gpx_to_geojson(gpx: String) -> Result<GeoJson> {
    let gpx_data = gpx::read(gpx.as_bytes())?;
    info!("number of tracks in gpx: {}", gpx_data.tracks.len());
    let geo_json: GeoJson = gpx_data
        .tracks
        .into_iter()
        .map(|track: gpx::Track| track.multilinestring().into_ride_feature())
        .collect::<Result<geojson::FeatureCollection>>()?
        .into();
    Ok(geo_json)
}
