use std::{collections::HashMap, sync::Arc};

use bigdecimal::BigDecimal;
use color_eyre::eyre::{eyre, Result};
use futures::stream::{self, TryStreamExt};
use google_maps::{prelude::*, LatLng};
use tokio::sync::Mutex;
use tracing::{debug, instrument};

use crate::{
    clients::get_google_maps,
    types::nominatim::{self, NominatimPlace},
};

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

struct WayData {
    seq: usize,
    address: HashMap<String, String>,
    surface: Option<String>,
}

// fn new_way_data(element: &OverpassElement) -> Result<WayData> {
//     Ok(WayData {
//         way: Way {
//             osm_id: element.id,
//             name: element.tags.name.ok_or(eyre!("No name on element!"))?,
//             surface_composition: element
//                 .tags
//                 .surface
//                 .ok_or(eyre!("No surface on element!"))?,
//         },
//         last_point: Point::default(),
//         distance: 0,
//     })
// }

#[instrument(skip(route))]
pub async fn ways(route: Vec<LatLng>, total_distance: BigDecimal) -> Result<Vec<WayData>> {
    let nominatimUrl = std::env::var("EXPEDITION_NOMINATIM_URL")?;
    let client = reqwest::Client::new();
    //Key is id. The decimal is distance on the way
    let ways = Arc::new(Mutex::new(HashMap::<usize, Vec<WayData>>::new()));
    stream::iter(route.iter().enumerate().map(Ok)).try_for_each_concurrent(10, |(seq, coord)| {
        let client = client.clone();
        let nominatimUrl = nominatimUrl.clone();
        let ways = ways.clone();
        async move {
            let place = client
                .get(format!(
                    "${nominatimUrl}/reverse?lat={lat}&lon={lon}&extratags=1&format=json",
                    lat = coord.lat,
                    lon = coord.lng
                ))
                .send()
                .await?
                .json::<NominatimPlace>()
                .await?;
            if place.osm_type == "way" {
                ways.lock()
                    .await
                    .entry(place.place_id)
                    .or_insert(Vec::new())
                    .push(WayData {
                        seq,
                        address: place.address,
                        surface: place.extratags.surface,
                    })
            }
            Ok::<(), color_eyre::eyre::Error>(())
        }
    });
    Ok(vec![])
}

fn aggregate_surface(surface: &str) -> &str {
    match surface {
        "gravel" | "unpaved" | "dirt" | "fine_gravel" | "rock" => "dirt",
        "asphalt" | "paved" => "tarmac",
        s => s,
    }
}
