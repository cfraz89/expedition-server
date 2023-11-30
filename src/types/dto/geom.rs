use google_maps::{prelude::Decimal, LatLng};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct PartialLatLng {
    pub lat: Option<Decimal>,
    pub lon: Option<Decimal>,
}

impl From<PartialLatLng> for Option<LatLng> {
    fn from(value: PartialLatLng) -> Self {
        match (value.lat, value.lon) {
            (Some(lat), Some(lon)) => LatLng::try_from_dec(lat, lon).ok(),
            _ => None,
        }
    }
}
