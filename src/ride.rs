use bigdecimal::BigDecimal;
use geo_types::Point;
use geojson::{Feature, FeatureCollection, Geometry};

use crate::{
    ride_geo::{Distance, EndPoint, Points, StartPoint},
    ride_processing::{reverse_geocode, ride_time, surface_composition},
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
    let start_latlng = google_maps::LatLng::try_from(&start_point)?;
    let end_latlng = google_maps::LatLng::try_from(&end_point)?;
    let (start_address, end_address, ride_time, surface_composition) = join!(
        reverse_geocode(start_latlng.to_owned()),
        reverse_geocode(end_latlng.to_owned()),
        ride_time(start_latlng, end_latlng),
        surface_composition(
            feature_collection
                .points()
                .map(|p| google_maps::LatLng::try_from(&p).unwrap())
                .collect()
        )
    );
    Ok(Ride {
        id: None,
        name,
        geo_json: sqlx::types::Json(feature_collection.into()),
        total_distance,
        surface_composition: sqlx::types::Json(surface_composition?),
        ride_time: ride_time?.num_seconds(),
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
