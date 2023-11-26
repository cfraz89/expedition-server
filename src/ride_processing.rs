use std::{collections::HashMap, sync::Arc};

use bigdecimal::BigDecimal;
use color_eyre::eyre::{eyre, Result};
use futures::stream::{self, TryStreamExt};
use geo::VincentyDistance;
use geo_types::Point;
use tokio::sync::Mutex;
use tracing::instrument;

use crate::{
    clients::{get_nominatim_url, get_reqwest_client},
    types::{
        nominatim::NominatimPlace,
        ride::{Way, WayPoint},
    },
};

#[instrument]
pub async fn nominatim_reverse_geocode(point: &Point) -> Result<NominatimPlace> {
    let url = format!(
        "{base_url}/reverse?lat={lat}&lon={lon}&extratags=1&format=json",
        base_url = get_nominatim_url()?,
        lat = point.y(),
        lon = point.x()
    );
    let place = get_reqwest_client()?
        .get(url)
        .send()
        .await?
        .json::<NominatimPlace>()
        .await?;
    Ok(place)
}

// #[instrument]
// pub async fn ride_time(start: LatLng, end: LatLng) -> Result<Duration> {
//     Ok(get_google_maps()?
//         .distance_matrix(vec![Waypoint::LatLng(start)], vec![Waypoint::LatLng(end)])
//         .execute()
//         .await?
//         .rows
//         .first()
//         .ok_or(eyre!("No row 0!"))?
//         .elements
//         .first()
//         .ok_or(eyre!("No element 0!"))?
//         .duration
//         .clone()
//         .ok_or(eyre!("No duration!"))?
//         .value)
// }

#[instrument(skip(route))]
pub async fn ways(
    route: impl Iterator<Item = Point> + Send,
    total_distance: &BigDecimal,
) -> Result<Vec<Way>> {
    //In parallel, get places from nominatim corresponding to coordinates, group them by place id
    let ways = Arc::new(Mutex::new(HashMap::<usize, Way>::new()));
    stream::iter(route.enumerate().map(Ok))
        .try_for_each_concurrent(50, |(seq, point)| {
            let ways = ways.clone();
            async move {
                let place = nominatim_reverse_geocode(&point).await?;
                if place.osm_type == "way" {
                    let mut ways = ways.lock().await;
                    let way = ways.entry(place.place_id).or_insert(Way {
                        distance: 0.0,
                        points: Vec::new(),
                        address: place.address,
                        surface: place.extratags.surface,
                    });
                    way.points.push(WayPoint { seq, point })
                }
                Ok::<(), color_eyre::eyre::Error>(())
            }
        })
        .await?;
    //Dont need our mutex anymore.
    let mut ways = Arc::into_inner(ways)
        .ok_or(eyre!("Couldnt unwrap arc!"))?
        .into_inner();
    //Now calculate the distance of each place.
    ways.iter_mut().for_each(|(_place_id, way)| {
        way.points.sort_by_key(|p| p.seq);
        way.distance = way
            .points
            .iter()
            .map_windows(|[p1, p2]| p1.point.vincenty_distance(&p2.point).unwrap_or(0.0))
            .sum();
    });
    let mut ways_vec = ways.into_values().collect::<Vec<Way>>();
    ways_vec.sort_by(|way1, way2| way1.distance.total_cmp(&way2.distance).reverse());
    Ok(ways_vec)
}

fn aggregate_surface(surface: &str) -> &str {
    match surface {
        "gravel" | "unpaved" | "dirt" | "fine_gravel" | "rock" => "dirt",
        "asphalt" | "paved" => "tarmac",
        s => s,
    }
}
