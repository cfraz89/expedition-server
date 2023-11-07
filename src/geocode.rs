use color_eyre::eyre::{eyre, Result};
use geo_types::Point;
use google_maps::{prelude::*, LatLng};

use crate::clients::get_google_maps;

pub fn lat_lng_from_point(point: Point) -> Result<LatLng, google_maps::GoogleMapsError> {
    LatLng::try_from_f64(point.y(), point.x())
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
