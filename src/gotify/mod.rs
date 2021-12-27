use super::alertmanager::models::Alert;
use super::config::models::Config;
use anyhow::Result;
use models::Notification;
use reqwest;

pub mod models;

pub fn gen_notification(alert: Alert) -> Notification {
    let mut notify = Notification {
        ..Default::default()
    };

    notify.title = alert.annotations.title.unwrap();
    notify.message = alert.annotations.description.unwrap();

    notify
}

pub async fn forward_notification(notify: Notification, config: &Config) -> Result<()> {
    let client = reqwest::Client::new();

    client
        .post(format!(
            "{}/message?token={}",
            config.gotify.uri, config.gotify.token
        ))
        .json(&notify)
        .send()
        .await?;

    Ok(())
}
