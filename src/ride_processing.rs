use std::collections::HashMap;

use bigdecimal::BigDecimal;
use color_eyre::eyre::{eyre, Result};
use google_maps::{prelude::*, LatLng};
use tracing::{debug, instrument};

use crate::{clients::get_google_maps, types::overpass::OverpassResponse};

#[instrument]
pub async fn reverse_geocode(latlng: LatLng) -> Result<Option<Geocoding>> {
    Ok(get_google_maps()?
        .reverse_geocoding(latlng)
        .execute()
        .await?
        .results
        .first()
        .cloned())
}

#[instrument]
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

#[instrument(skip(route))]
pub async fn surface_composition(route: Vec<LatLng>) -> Result<HashMap<String, BigDecimal>> {
    let client = reqwest::Client::new();
    let mut surface_map = HashMap::<String, i64>::new();
    let resp = client
        .post("https://overpass-api.de/api/interpreter")
        .body(format!(
            r#"
                    [out:json];
                    ({});
                    out tags;
                    "#,
            route
                .iter()
                .map(|coord| format!("way[highway](around: 1, {}, {});", coord.lat, coord.lng))
                .collect::<String>()
        ))
        .send()
        .await?
        .json::<OverpassResponse>()
        .await?;
    resp.elements.iter().for_each(|el| {
        if let Some(surface) = &el.tags.surface {
            surface_map
                .entry(aggregate_surface(surface).to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    });
    debug!("Surface map: {:?}", surface_map);
    let mut surface_ratio_map = HashMap::<String, BigDecimal>::new();
    let total_samples: i64 = surface_map.values().sum();
    surface_map.iter().for_each(|(k, v)| {
        surface_ratio_map
            .entry(k.to_string())
            .or_insert(BigDecimal::from(*v) / BigDecimal::from(total_samples));
    });
    debug!("Surface ratio map: {:?}", surface_ratio_map);
    Ok(surface_ratio_map)
}

fn aggregate_surface(surface: &str) -> &str {
    match surface {
        "gravel" | "unpaved" | "dirt" | "fine_gravel" | "rock" => "dirt",
        "asphalt" | "paved" => "tarmac",
        s => s,
    }
}
