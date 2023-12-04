use color_eyre::eyre::Result;

use crate::ride_geo::{BoundingBox, Distance};

use geojson::FeatureCollection;
use geojson::{Feature, Geometry};
use gpx::Gpx;
use gpx::Track;
use tracing::info;

use crate::types::feature::FeatureProperties;

pub trait AsRideFeatureCollection {
    fn as_ride_feature_collection(&self) -> Result<FeatureCollection>;
}

impl AsRideFeatureCollection for Gpx {
    fn as_ride_feature_collection(&self) -> Result<FeatureCollection> {
        info!("number of tracks in gpx: {}", self.tracks.len());
        Ok(self
            .tracks
            .iter()
            .filter_map(|track| track.as_ride_feature())
            .collect())
    }
}

pub trait AsRideFeature {
    fn as_ride_feature(&self) -> Option<Feature>;
}

impl AsRideFeature for Track {
    fn as_ride_feature(&self) -> Option<Feature> {
        let mls = self.multilinestring();
        let num_points: usize = mls.iter().map(|ls| ls.0.len()).sum();
        // Become empty on tracks which for some reason have no points, it'll ruin the collected bounding box
        if num_points == 0 {
            return None;
        }

        let bounding_box = mls.bounding_box();
        let geom = Geometry {
            bbox: bounding_box.to_owned(),
            value: (&mls).into(),
            foreign_members: None,
        };
        let distance = geom.distance();
        return Some(Feature {
            bbox: bounding_box.to_owned(),
            geometry: Some(geom),
            properties: Some(
                FeatureProperties {
                    distance,
                    name: self.name.clone(),
                }
                .try_into()
                .expect("Shouldnt fail json conversion"),
            ),
            ..Default::default()
        });
    }
}
