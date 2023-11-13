use bigdecimal::BigDecimal;
use geo_types::Point;
use geojson::{Feature, FeatureCollection, Geometry};

use crate::{
    geocode::reverse_geocode,
    ride_geo::{Distance, EndPoint, StartPoint},
    types::ride::Ride,
};
use color_eyre::eyre::{eyre, Result};
use futures::join;

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
    let (start_address, end_address) = join!(
        reverse_geocode(google_maps::LatLng::try_from(&start_point)?.to_owned()),
        reverse_geocode(google_maps::LatLng::try_from(&end_point)?.to_owned())
    );
    Ok(Ride {
        id: None,
        name,
        geo_json: sqlx::types::Json(feature_collection.into()),
        total_distance,
        start_address: sqlx::types::Json(
            start_address?.map_or(vec![], |addr| addr.address_components),
        ),
        end_address: sqlx::types::Json(end_address?.map_or(vec![], |addr| addr.address_components)),
    })
}

fn feature_point(id: String, point: &Point) -> Feature {
    Feature {
        id: Some(geojson::feature::Id::String(id)),
        geometry: Some(Geometry::new(point.into())),
        ..Default::default()
    }
}
