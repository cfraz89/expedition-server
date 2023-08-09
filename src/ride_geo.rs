use color_eyre::eyre::Result;
use geo::{BoundingRect, VincentyDistance};
use geo_types::{CoordFloat, CoordNum, LineString, MultiLineString, MultiPoint, Point};
use geojson::{Feature, GeoJson};

use crate::types::feature::FeatureProperties;

pub trait IntoRideGeoJson<'a> {
    fn into_ride_geo_json(&'a self) -> Result<GeoJson>;
}

pub trait IntoRideFeature<'a> {
    fn into_ride_feature(&'a self) -> Result<Feature>;
}

impl<'a, S> IntoRideFeature<'a> for S
where
    S: BoundingBox<f64> + 'a,
    &'a S: Into<geojson::Value>,
{
    fn into_ride_feature(&'a self) -> Result<Feature> {
        let bounding_box = self.bounding_box();
        let geom = geojson::Geometry {
            bbox: bounding_box.to_owned(),
            value: <&'a S as Into<geojson::Value>>::into(self),
            foreign_members: None,
        };
        let distance = geom.distance();
        return Ok(Feature {
            bbox: bounding_box.to_owned(),
            geometry: Some(geom),
            properties: Some(FeatureProperties { distance }.try_into()?),
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

impl<N> Distance<N> for MultiPoint<N>
where
    N: std::iter::Sum + CoordFloat,
    Point<N>: VincentyDistance<N>,
{
    fn distance(&self) -> N {
        self.0
            .windows(2)
            .filter_map(|p| p[0].vincenty_distance(&p[1]).ok())
            .sum()
    }
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

impl Distance<f64> for geojson::Value {
    fn distance(&self) -> f64 {
        match self {
            geojson::Value::Point(_) => 0.0,
            geojson::Value::MultiPoint(_) => {
                geo_types::MultiPoint::try_from(self).unwrap().distance()
            }
            geojson::Value::LineString(_) => {
                geo_types::LineString::try_from(self).unwrap().distance()
            }
            geojson::Value::MultiLineString(_) => geo_types::MultiLineString::try_from(self)
                .unwrap()
                .distance(),
            geojson::Value::Polygon(_) => 0.0,
            geojson::Value::MultiPolygon(_) => 0.0,
            geojson::Value::GeometryCollection(geoms) => {
                geoms.iter().map(|geom| geom.value.distance()).sum()
            }
        }
    }
}

impl Distance<f64> for geojson::Geometry {
    fn distance(&self) -> f64 {
        self.value.distance()
    }
}

impl Distance<f64> for geojson::Feature {
    fn distance(&self) -> f64 {
        self.geometry.as_ref().map_or(0.0, |geom| geom.distance())
    }
}

impl Distance<f64> for geojson::FeatureCollection {
    fn distance(&self) -> f64 {
        self.into_iter().map(|feat| feat.distance()).sum()
    }
}

impl Distance<f64> for GeoJson {
    fn distance(&self) -> f64 {
        match self {
            GeoJson::Geometry(geom) => geom.distance(),
            GeoJson::Feature(feat) => feat.distance(),
            GeoJson::FeatureCollection(fc) => fc.distance(),
        }
    }
}
