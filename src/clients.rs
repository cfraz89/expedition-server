use std::sync::OnceLock;

use color_eyre::eyre::{eyre, Result};
use google_maps::GoogleMapsClient;
use sqlx::{Pool, Postgres};

pub static DB_POOL: OnceLock<Pool<Postgres>> = OnceLock::new();
pub static GMAPS: OnceLock<GoogleMapsClient> = OnceLock::new();

pub fn get_db_pool() -> Result<&'static Pool<Postgres>> {
    DB_POOL.get().ok_or(eyre!("Failed to get db"))
}

pub fn get_google_maps() -> Result<&'static GoogleMapsClient> {
    GMAPS.get().ok_or(eyre!("Failed to get google maps"))
}
