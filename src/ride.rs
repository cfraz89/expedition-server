use bigdecimal::BigDecimal;
use geo_types::Point;
use geojson::{Feature, FeatureCollection, Geometry};

use crate::{
    ride_geo::{Distance, EndPoint, Points, StartPoint},
    ride_processing::ride_ways,
    types::model::ride::Ride,
};
use color_eyre::eyre::{eyre, Result};

pub async fn create_ride(name: String, mut feature_collection: FeatureCollection) -> Result<Ride> {
    let start_point = feature_collection
        .start_point()
        .ok_or(eyre!("No start point on geometry"))?;
    let end_point = feature_collection
        .end_point()
        .ok_or(eyre!("No end point on geometry"))?;
    feature_collection
        .features
        .push(feature_point(String::from("start"), &start_point));
    feature_collection
        .features
        .push(feature_point(String::from("end"), &end_point));
    let total_distance = BigDecimal::try_from(feature_collection.distance())?;
    let ways = ride_ways(feature_collection.points(), &total_distance).await?;
    Ok(Ride {
        id: None,
        name,
        geo_json: sqlx::types::Json(feature_collection.into()),
        total_distance,
        ways: sqlx::types::Json(ways),
    })
}

fn feature_point(id: String, point: &Point) -> Feature {
    Feature {
        id: Some(geojson::feature::Id::String(id)),
        geometry: Some(Geometry::new(point.into())),
        ..Default::default()
    }
}
