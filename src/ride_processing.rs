use std::{collections::HashMap, sync::Arc};

use bigdecimal::BigDecimal;
use color_eyre::eyre::{eyre, Result};
use futures::{
    stream::{self, StreamExt, TryStreamExt},
    TryFutureExt,
};
use google_maps::{prelude::*, LatLng};
use tokio::sync::Mutex;

use crate::{clients::get_google_maps, types::overpass::OverpassResponse};

pub async fn reverse_geocode(latlng: LatLng) -> Result<Option<Geocoding>> {
    Ok(get_google_maps()?
        .reverse_geocoding(latlng)
        .execute()
        .await?
        .results
        .first()
        .cloned())
}

pub async fn ride_time(start: LatLng, end: LatLng) -> Result<Duration> {
    Ok(get_google_maps()?
        .distance_matrix(vec![Waypoint::LatLng(start)], vec![Waypoint::LatLng(end)])
        .execute()
        .await?
        .rows
        .first()
        .ok_or(eyre!("No row 0!"))?
        .elements
        .first()
        .ok_or(eyre!("No element 0!"))?
        .duration
        .clone()
        .ok_or(eyre!("No duration!"))?
        .value)
}

pub async fn surface_composition(route: Vec<LatLng>) -> Result<HashMap<String, BigDecimal>> {
    let client = reqwest::Client::new();
    let surface_map = Arc::new(Mutex::new(HashMap::<String, i64>::new()));
    stream::iter(route)
        .map(|latlng| {
            client
                .post("https://overpass-api.de/api/interpreter")
                .body(format!(
                    r#"
                    [out:json];
                    way[highway](around: 1, {}, {});
                    out geom;
                    "#,
                    latlng.lat, latlng.lng
                ))
                .send()
                .and_then(|r| async move { r.json::<OverpassResponse>().await })
        })
        .buffer_unordered(10)
        .try_for_each(|resp| async {
            let surface_map_ref = surface_map.clone();
            move || {
                resp.elements.iter().for_each(|el| {
                    if let Some(surface) = &el.tags.surface {
                        surface_map_ref
                            .blocking_lock()
                            .entry(surface.to_string())
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    }
                })
            };
            Ok(())
        })
        .await?;
    let surface_map = surface_map.lock().await;
    let mut surface_ratio_map = HashMap::<String, BigDecimal>::new();
    let total_samples: i64 = surface_map.values().sum();
    surface_map.iter().for_each(|(k, v)| {
        surface_ratio_map
            .entry(k.to_string())
            .or_insert(BigDecimal::from(*v) / BigDecimal::from(total_samples));
    });
    Ok(surface_ratio_map)
}
