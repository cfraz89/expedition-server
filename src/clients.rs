use std::sync::OnceLock;

use color_eyre::eyre::{eyre, Result};
use google_maps::GoogleMapsClient;
use surrealdb::{engine::remote::ws::Client, Surreal};

pub static DB: OnceLock<Surreal<Client>> = OnceLock::new();
pub static GMAPS: OnceLock<GoogleMapsClient> = OnceLock::new();

pub fn get_db() -> Result<&'static Surreal<Client>> {
    DB.get().ok_or(eyre!("Failed to get db"))
}

pub fn get_google_maps() -> Result<&'static GoogleMapsClient> {
    GMAPS.get().ok_or(eyre!("Failed to get google maps"))
}
