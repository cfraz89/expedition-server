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
            .filter_map(|track| {
                let ms = track.multilinestring();
                let num_points: usize = ms.iter().map(|ls| ls.0.len()).sum();
                // Dont include tracks which for some reason have no points, it'll ruin the collected bounding box
                if num_points > 0 {
                    Some(ms.into_ride_feature())
                } else {
                    None
                }
            })
            .collect::<Result<geojson::FeatureCollection>>()?
            .into();
        Ok(geo_json)
    }
}
