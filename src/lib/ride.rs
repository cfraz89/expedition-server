use color_eyre::eyre::Result;
use geo::{BoundingRect, VincentyDistance};
use geo_types::{CoordFloat, CoordNum, LineString, MultiLineString, Point};
use geojson::Feature;

use crate::types::feature::FeatureProperties;

pub trait IntoRideFeature<'a> {
    fn into_ride_feature(&'a self) -> Result<Feature>;
}

impl<'a, S> IntoRideFeature<'a> for S
where
    S: BoundingBox<f64> + Distance<f64>,
    &'a S: Into<geojson::Value> + 'a,
{
    fn into_ride_feature(&'a self) -> Result<Feature> {
        let bounding_box = self.bounding_box();
        return Ok(Feature {
            bbox: bounding_box.to_owned(),
            geometry: Some(geojson::Geometry {
                bbox: bounding_box.to_owned(),
                value: <&'a S as Into<geojson::Value>>::into(self),
                foreign_members: None,
            }),
            properties: Some(
                FeatureProperties {
                    distance: self.distance(),
                }
                .try_into()?,
            ),
            ..Default::default()
        });
    }
}

//Get the bounding box for a geometry as a vector
pub trait BoundingBox<N> {
    fn bounding_box(&self) -> Option<Vec<N>>;
}

impl<T, N> BoundingBox<N> for T
where
    T: BoundingRect<N>,
    N: CoordNum,
{
    fn bounding_box(&self) -> Option<Vec<N>> {
        self.bounding_rect()
            .into()
            .map(|r| vec![r.min().x, r.min().y, r.max().x, r.max().y])
    }
}

/// Calculate distance of all points in multilinestring, excluding point pairs with incalculable distance
pub trait Distance<N> {
    fn distance(&self) -> N;
}

impl<N> Distance<N> for LineString<N>
where
    N: std::iter::Sum + CoordFloat,
    Point<N>: VincentyDistance<N>,
{
    fn distance(&self) -> N {
        self.points()
            .collect::<Vec<Point<N>>>()
            .windows(2)
            .filter_map(|p| p[0].vincenty_distance(&p[1]).ok())
            .sum()
    }
}

impl<N> Distance<N> for MultiLineString<N>
where
    N: std::iter::Sum + CoordFloat,
    Point<N>: VincentyDistance<N>,
{
    fn distance(&self) -> N {
        self.iter().map(|line| line.distance()).sum()
    }
}
