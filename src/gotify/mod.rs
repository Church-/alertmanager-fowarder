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

    if let Some(title) = alert.annotations.title {
        notify.title = title;
    }
    if let Some(description) = alert.annotations.description {
        notify.message = description;
    }
    if let Some(priority) = alert.annotations.priority {
        if let Ok(priority) = priority.parse() {
            notify.priority = priority;
        }
    }

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
