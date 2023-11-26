use color_eyre::eyre::Result;
use geo::{BoundingRect, VincentyDistance};
use geo_types::{CoordFloat, CoordNum, LineString, MultiLineString, MultiPoint, Point};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry};

use crate::types::feature::FeatureProperties;

pub trait IntoRideFeatureCollection<'a> {
    fn into_ride_feature_collection(&'a self) -> Result<FeatureCollection>;
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
        let geom = Geometry {
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

impl Distance<f64> for Geometry {
    fn distance(&self) -> f64 {
        self.value.distance()
    }
}

impl Distance<f64> for Feature {
    fn distance(&self) -> f64 {
        self.geometry.as_ref().map_or(0.0, |geom| geom.distance())
    }
}

impl Distance<f64> for FeatureCollection {
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

pub trait StartPoint {
    fn start_point(&self) -> Option<Point>;
}

impl StartPoint for geojson::Value {
    fn start_point(&self) -> Option<Point> {
        match self {
            geojson::Value::Point(_) => Some(geo_types::Point::try_from(self).unwrap()),
            geojson::Value::MultiPoint(_) => geo_types::MultiPoint::try_from(self)
                .unwrap()
                .0
                .clone()
                .first()
                .copied(),
            geojson::Value::LineString(_) => geo_types::LineString::try_from(self)
                .unwrap()
                .points()
                .next(),
            geojson::Value::MultiLineString(_) => geo_types::MultiLineString::try_from(self)
                .unwrap()
                .iter()
                .next()?
                .points()
                .next(),
            geojson::Value::Polygon(_) => None,
            geojson::Value::MultiPolygon(_) => None,
            geojson::Value::GeometryCollection(geoms) => geoms.iter().next()?.start_point(),
        }
    }
}

impl StartPoint for Geometry {
    fn start_point(&self) -> Option<Point> {
        self.value.start_point()
    }
}

impl StartPoint for Feature {
    fn start_point(&self) -> Option<Point> {
        let geo = self.geometry.as_ref()?;
        geo.start_point()
    }
}

impl StartPoint for FeatureCollection {
    fn start_point(&self) -> Option<Point> {
        self.into_iter().next()?.start_point()
    }
}

impl StartPoint for GeoJson {
    fn start_point(&self) -> Option<Point> {
        match self {
            GeoJson::Geometry(geom) => geom.start_point(),
            GeoJson::Feature(feat) => feat.start_point(),
            GeoJson::FeatureCollection(fc) => fc.start_point(),
        }
    }
}

pub trait EndPoint {
    fn end_point(&self) -> Option<Point>;
}

impl EndPoint for geojson::Value {
    fn end_point(&self) -> Option<Point> {
        match self {
            geojson::Value::Point(_) => Some(geo_types::Point::try_from(self).unwrap()),
            geojson::Value::MultiPoint(_) => geo_types::MultiPoint::try_from(self)
                .unwrap()
                .0
                .clone()
                .last()
                .copied(),
            geojson::Value::LineString(_) => geo_types::LineString::try_from(self)
                .unwrap()
                .points()
                .last(),
            geojson::Value::MultiLineString(_) => geo_types::MultiLineString::try_from(self)
                .unwrap()
                .iter()
                .last()?
                .points()
                .last(),
            geojson::Value::Polygon(_) => None,
            geojson::Value::MultiPolygon(_) => None,
            geojson::Value::GeometryCollection(geoms) => geoms.iter().last()?.end_point(),
        }
    }
}

impl EndPoint for Geometry {
    fn end_point(&self) -> Option<Point> {
        self.value.end_point()
    }
}

impl EndPoint for Feature {
    fn end_point(&self) -> Option<Point> {
        let geo = self.geometry.as_ref()?;
        geo.end_point()
    }
}

impl EndPoint for FeatureCollection {
    fn end_point(&self) -> Option<Point> {
        self.into_iter().next()?.end_point()
    }
}

impl EndPoint for GeoJson {
    fn end_point(&self) -> Option<Point> {
        match self {
            GeoJson::Geometry(geom) => geom.end_point(),
            GeoJson::Feature(feat) => feat.end_point(),
            GeoJson::FeatureCollection(fc) => fc.end_point(),
        }
    }
}

pub trait Points {
    fn points(&self) -> impl Iterator<Item = Point> + Send;
}

impl Points for geojson::Value {
    fn points(&self) -> Box<dyn Iterator<Item = Point> + '_ + Send> {
        match self {
            geojson::Value::Point(_) => {
                Box::new(std::iter::once(geo_types::Point::try_from(self).unwrap()))
            }
            geojson::Value::MultiPoint(_) => {
                Box::new(geo_types::MultiPoint::try_from(self).unwrap().0.into_iter())
            }
            geojson::Value::LineString(_) => Box::new(
                geo_types::LineString::try_from(self)
                    .unwrap()
                    .into_points()
                    .into_iter(),
            ),
            geojson::Value::MultiLineString(_) => Box::new(
                geo_types::MultiLineString::try_from(self)
                    .unwrap()
                    .into_iter()
                    .flat_map(|ls| ls.points().collect::<Vec<_>>()),
            ),
            geojson::Value::Polygon(_) => Box::new(std::iter::empty()),
            geojson::Value::MultiPolygon(_) => Box::new(std::iter::empty()),
            geojson::Value::GeometryCollection(geoms) => {
                Box::new(geoms.into_iter().flat_map(|g| g.points()))
            }
        }
    }
}

impl Points for Geometry {
    fn points(&self) -> impl Iterator<Item = Point> + Send {
        self.value.points()
    }
}

impl Points for Feature {
    fn points(&self) -> Box<dyn Iterator<Item = Point> + '_ + Send> {
        match &self.geometry {
            None => Box::new(std::iter::empty()),
            Some(geo) => Box::new(geo.points()),
        }
    }
}

impl Points for FeatureCollection {
    fn points(&self) -> impl Iterator<Item = Point> + Send {
        self.into_iter().flat_map(|f| f.points())
    }
}

impl Points for GeoJson {
    fn points(&self) -> Box<dyn Iterator<Item = Point> + '_ + Send> {
        match self {
            GeoJson::Geometry(geom) => Box::new(geom.points()),
            GeoJson::Feature(feat) => Box::new(feat.points()),
            GeoJson::FeatureCollection(fc) => Box::new(fc.points()),
        }
    }
}
