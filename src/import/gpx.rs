use color_eyre::Result;
use geojson::FeatureCollection;
use gpx::Gpx;
use tracing::{info, instrument};

use crate::ride_geo::{IntoRideFeature, IntoRideFeatureCollection};

impl<'a> IntoRideFeatureCollection<'a> for Gpx {
    fn into_ride_feature_collection(&'a self) -> Result<FeatureCollection> {
        info!("number of tracks in gpx: {}", self.tracks.len());
        self.tracks
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
            .collect()
    }
}
