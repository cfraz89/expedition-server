use color_eyre::eyre::Result;
use google_maps::{prelude::*, LatLng};

use crate::clients::get_google_maps;

pub async fn reverse_geocode(latlng: LatLng) -> Result<Option<Geocoding>> {
    Ok(get_google_maps()?
        .reverse_geocoding(latlng)
        .execute()
        .await?
        .results
        .first()
        .cloned())
}
