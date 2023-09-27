use color_eyre::eyre::{eyre, Result};
use geo_types::Point;
use google_maps::{
    prelude::{Decimal, Geocoding},
    LatLng,
};

use crate::clients::get_google_maps;

pub fn lat_lng_from_point(point: Point) -> LatLng {
    LatLng {
        lat: Decimal::from_f64_retain(point.y()).unwrap(),
        lng: Decimal::from_f64_retain(point.x()).unwrap(),
    }
}

pub async fn reverse_geocode(latlng: LatLng) -> Result<Option<Geocoding>> {
    Ok(get_google_maps()?
        .reverse_geocoding(latlng)
        .execute()
        .await?
        .results
        .first()
        .cloned())
}
