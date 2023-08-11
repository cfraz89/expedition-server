use geo_types::Point;
use geojson::{Feature, FeatureCollection, Geometry};
use google_maps::{
    prelude::{Decimal, Geocoding},
    LatLng,
};

use crate::{
    clients::get_google_maps,
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

    let total_distance = feature_collection.distance();
    let (start_address, end_address) = join!(
        reverse_geocode(lat_lng_from_point(start_point)),
        reverse_geocode(lat_lng_from_point(end_point))
    );
    Ok(Ride {
        id: None,
        name,
        geo_json: feature_collection.into(),
        total_distance,
        start_address: start_address?.map_or(vec![], |addr| addr.address_components),
        end_address: end_address?.map_or(vec![], |addr| addr.address_components),
    })
}

fn feature_point(id: String, point: &Point) -> Feature {
    Feature {
        id: Some(geojson::feature::Id::String(id)),
        geometry: Some(Geometry::new(point.into())),
        ..Default::default()
    }
}

fn lat_lng_from_point(point: Point) -> LatLng {
    LatLng {
        lat: Decimal::from_f64_retain(point.y()).unwrap(),
        lng: Decimal::from_f64_retain(point.x()).unwrap(),
    }
}

async fn reverse_geocode(latlng: LatLng) -> Result<Option<Geocoding>> {
    Ok(get_google_maps()?
        .reverse_geocoding(latlng)
        .execute()
        .await?
        .results
        .first()
        .cloned())
}
