use std::{collections::HashMap, sync::Arc};

use bigdecimal::BigDecimal;
use color_eyre::eyre::{eyre, Result};
use futures::stream::{self, TryStreamExt};
use geo::VincentyDistance;
use geo_types::Point;
use google_maps::{prelude::*, LatLng};
use tokio::{sync::Mutex, try_join};
use tracing::{debug, info, instrument};

use crate::{
    clients::{get_google_maps, get_nominatim_url, get_reqwest_client},
    types::{
        dto::nominatim::{NominatimDetailsPlace, NominatimPlace},
        model::{
            self,
            ride::{ProcessedRide, RideWay, WayPoint},
        },
    },
};

pub async fn nominatim_reverse_geocode(point: &Point) -> Result<NominatimPlace> {
    let url = format!(
        "{base_url}/reverse?lat={lat}&lon={lon}&extratags=1&format=jsonv2",
        base_url = get_nominatim_url()?,
        lat = point.y(),
        lon = point.x()
    );
    let place = get_reqwest_client()?
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<NominatimPlace>()
        .await?;
    Ok(place)
}

pub async fn nominatim_get_place(osm_type: &str, osm_id: u64) -> Result<NominatimDetailsPlace> {
    let url = format!(
        "{base_url}/details?osmtype={osm_type}&osmid={osm_id}&addressdetails=1&format=json",
        base_url = get_nominatim_url()?
    );
    let place = get_reqwest_client()?
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json::<NominatimDetailsPlace>()
        .await?;
    Ok(place)
}

#[instrument(skip(route))]
pub async fn ride_ways(
    route: impl Iterator<Item = Point> + Send,
    total_distance: &BigDecimal,
) -> Result<Vec<RideWay>> {
    //In parallel, get places from nominatim corresponding to coordinates, group them by place id
    let ways = Arc::new(Mutex::new(HashMap::<String, RideWay>::new()));
    stream::iter(route.enumerate().map(Ok))
        .try_for_each_concurrent(50, |(seq, point)| {
            let ways = ways.clone();
            async move {
                let place = nominatim_reverse_geocode(&point).await?;
                if place.osm_type == "way" && place.category == Some(String::from("highway")) {
                    if let Some(place_name) = place.name {
                        let mut ways = ways.lock().await;
                        //We use name, not id, as the hash to group by, as the same road can have multiple place ids
                        let way = ways.entry(place_name).or_insert(RideWay {
                            seq: seq.try_into().expect("Couldn't convert usize to u64"),
                            osm_id: place.osm_id,
                            distance: 0.0,
                            points: Vec::new(),
                        });
                        way.points.push(WayPoint { seq, point })
                    }
                }
                Ok::<(), color_eyre::eyre::Error>(())
            }
        })
        .await?;
    //Dont need our mutex anymore.
    let mut ways = Arc::into_inner(ways)
        .expect("Couldnt unwrap arc!")
        .into_inner();
    //Now calculate the distance of each place.
    ways.values_mut().for_each(|way| {
        way.points.sort_by_key(|p| p.seq);
        way.distance = way
            .points
            .iter()
            .map_windows(|[p1, p2]| p1.point.vincenty_distance(&p2.point).unwrap_or(0.0))
            .sum();
    });
    let mut ways_vec = ways.into_values().collect::<Vec<RideWay>>();
    // ways_vec.sort_by(|way1, way2| way1.distance.total_cmp(&way2.distance).reverse());
    ways_vec.sort_unstable_by_key(|way| way.seq);
    Ok(ways_vec)
}

pub fn aggregate_surface(surface: &str) -> &str {
    match surface {
        "gravel" | "unpaved" | "dirt" | "fine_gravel" | "rock" => "dirt",
        "asphalt" | "paved" => "tarmac",
        s => s,
    }
}

#[instrument]
pub async fn time_to_start_and_from_end(
    origin: LatLng,
    start: LatLng,
    end: LatLng,
) -> Result<(Duration, Duration)> {
    let distances = get_google_maps()?
        .distance_matrix(
            vec![Waypoint::LatLng(origin.clone()), Waypoint::LatLng(end)],
            vec![Waypoint::LatLng(start), Waypoint::LatLng(origin)],
        )
        .execute()
        .await?;

    let origin_to_start = distances
        .rows
        .first()
        .ok_or(eyre!("No row 0!"))?
        .elements
        .first()
        .ok_or(eyre!("No element 0!"))?
        .duration
        .clone()
        .ok_or(eyre!("No duration 0!"))?
        .value;

    let end_to_origin = distances
        .rows
        .get(1)
        .ok_or(eyre!("No row 1!"))?
        .elements
        .get(1)
        .ok_or(eyre!("No element 1!"))?
        .duration
        .clone()
        .ok_or(eyre!("No duration 1!"))?
        .value;

    Ok((origin_to_start, end_to_origin))
}

pub async fn process_ride(
    ride: model::ride::QueryRide,
    origin: Option<LatLng>,
) -> Result<ProcessedRide> {
    let start_point = ride.start_point.ok_or(eyre!("No start point"))?.0;
    let end_point = ride.end_point.ok_or(eyre!("No end point"))?.0;
    let start_coords = LatLng::try_from(&start_point)?;
    let end_coords = LatLng::try_from(&end_point)?;
    let time_fut = async move {
        if let Some(origin) = origin {
            let res = time_to_start_and_from_end(origin, start_coords, end_coords).await?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    };
    let (start_address, end_address, times) = try_join!(
        nominatim_reverse_geocode(&start_point),
        nominatim_reverse_geocode(&end_point),
        time_fut
    )?;
    Ok(model::ride::ProcessedRide {
        id: ride.id,
        name: ride.name,
        start_address: start_address.address.into(),
        end_address: end_address.address.into(),
        total_distance: ride.total_distance,
        time_from_origin_to_start: times.map(|t| t.0.num_seconds()),
        time_from_end_to_origin: times.map(|t| t.1.num_seconds()),
        start_point,
        end_point,
        geo_json: ride.geo_json,
        ways: ride.ways,
    })
}
