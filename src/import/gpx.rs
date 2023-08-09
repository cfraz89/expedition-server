use color_eyre::Result;
use geojson::GeoJson;
use gpx::Gpx;
use tracing::{info, instrument};

use crate::ride_geo::{IntoRideFeature, IntoRideGeoJson};

impl<'a> IntoRideGeoJson<'a> for Gpx {
    #[instrument]
    fn into_ride_geo_json(&'a self) -> Result<GeoJson> {
        // let gpx_data = gpx::read(gpx.as_bytes())?;
        info!("number of tracks in gpx: {}", self.tracks.len());
        let geo_json: GeoJson = self
            .tracks
            .iter()
            .map(|track| track.multilinestring().into_ride_feature())
            .collect::<Result<geojson::FeatureCollection>>()?
            .into();
        Ok(geo_json)
    }
}
