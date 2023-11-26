use std::{env::VarError, sync::OnceLock};

use color_eyre::eyre::{eyre, Result};
use sqlx::{Pool, Postgres};

pub static DB_POOL: OnceLock<Pool<Postgres>> = OnceLock::new();
pub static REQWEST: OnceLock<reqwest::Client> = OnceLock::new();
pub static NOMINATIM_URL: OnceLock<String> = OnceLock::new();

pub fn get_db_pool() -> Result<&'static Pool<Postgres>> {
    DB_POOL.get().ok_or(eyre!("Failed to get db"))
}

pub fn get_reqwest_client() -> Result<&'static reqwest::Client> {
    REQWEST.get().ok_or(eyre!("Failed to get reqwest client"))
}

pub fn get_nominatim_url() -> Result<&'static String> {
    NOMINATIM_URL
        .get()
        .ok_or(eyre!("Failed to get nominatim url"))
}
