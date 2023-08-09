use std::future::IntoFuture;

use geo_types::Point;
use geojson::GeoJson;
use google_maps::prelude::{Decimal, Geocoding};

use crate::{
    clients::get_google_maps,
    ride_geo::{Distance, EndPoint, StartPoint},
    types::ride::Ride,
};
use color_eyre::eyre::Result;
use futures::future::{self, FutureExt};
use futures::join;

pub async fn create_ride(name: String, geo_json: GeoJson) -> Result<Ride> {
    let total_distance = geo_json.distance();
    let start_address_future = match geo_json.start_point() {
        Some(p) => reverse_geocode_point(p).boxed(),
        None => future::ok(None).boxed(),
    };
    let end_address_future = match geo_json.end_point() {
        Some(p) => reverse_geocode_point(p).boxed(),
        None => future::ok(None).boxed(),
    };
    let (start_address, end_address) = join!(start_address_future, end_address_future);
    Ok(Ride {
        id: None,
        name,
        geo_json,
        total_distance,
        start_address: start_address?,
        end_address: end_address?,
    })
}

async fn reverse_geocode_point(point: Point) -> Result<Option<Geocoding>> {
    Ok(get_google_maps()?
        .reverse_geocoding(google_maps::LatLng {
            lat: Decimal::from_f64_retain(point.y()).unwrap(),
            lng: Decimal::from_f64_retain(point.x()).unwrap(),
        })
        .execute()
        .await?
        .results
        .first()
        .cloned())
}
