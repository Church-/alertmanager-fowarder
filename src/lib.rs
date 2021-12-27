#[macro_use]
extern crate rocket;

extern crate anyhow;
extern crate tokio;

use anyhow::Result;
use clap::{App, Arg};
use config::models::Config;
use rocket::{Build, Rocket};
use routes::*;

pub mod alertmanager;
pub mod config;
pub mod gotify;
pub mod logger;
pub mod routes;

pub async fn rocket() -> Result<Rocket<Build>> {
    let matches = App::new("Alert Manager Gotify Integration")
        .version("1.0")
        .author("Church")
        .about("Webhook integration for routing prometheus alerts.")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("config")
                .help("Path to the config file.")
                .takes_value(true),
        )
        .get_matches();

    let config_file = matches
        .value_of("config")
        .expect("Failed to find config file.");
    let config_str = std::fs::read_to_string(config_file)?;
    let config: Config = toml::from_str(&config_str)?;

    Ok(rocket::build()
        .attach(logger::log_requests())
        .manage(config)
        .mount("/", routes![route_prom_alert, get_health]))
}
