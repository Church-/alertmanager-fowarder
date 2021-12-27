use super::alertmanager::models::Payload;
use super::config::models::Config;
use super::gotify::{forward_notification, gen_notification};
use anyhow::Result;
use rocket::serde::json::Json;
use rocket::State;

#[post("/forward_alert", format = "application/json", data = "<payload>")]
pub async fn route_prom_alert(
    payload: Json<Payload>,
    config: &State<Config>,
) -> Result<(), rocket::response::Debug<anyhow::Error>> {
    for alert in payload.into_inner().alerts {
        dbg!(&alert);
        let notification = gen_notification(alert);
        forward_notification(notification, config).await?;
    }

    Ok(())
}

#[get("/healthz")]
pub async fn get_health() -> Result<&'static str, rocket::response::Debug<anyhow::Error>> {
    Ok("Good")
}
