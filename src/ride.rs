use bigdecimal::BigDecimal;
use geo_types::Point;
use geojson::{Feature, FeatureCollection, Geometry};

use crate::{
    ride_geo::{Distance, EndPoint, Points, StartPoint},
    ride_processing::{nominatim_reverse_geocode, ways},
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
    // let start_latlng = google_maps::LatLng::try_from(&start_point)?;
    // let end_latlng = google_maps::LatLng::try_from(&end_point)?;
    let (start_place, end_place, ways) = join!(
        nominatim_reverse_geocode(&start_point),
        nominatim_reverse_geocode(&end_point),
        ways(feature_collection.points(), &total_distance)
    );
    Ok(Ride {
        id: None,
        name,
        geo_json: sqlx::types::Json(feature_collection.into()),
        total_distance,
        start_address: sqlx::types::Json(start_place?.address),
        end_address: sqlx::types::Json(end_place?.address),
        ways: sqlx::types::Json(ways?),
    })
}

fn feature_point(id: String, point: &Point) -> Feature {
    Feature {
        id: Some(geojson::feature::Id::String(id)),
        geometry: Some(Geometry::new(point.into())),
        ..Default::default()
    }
}
