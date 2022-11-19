use anyhow::Result;
use serde::Serialize;

use crate::alertmanager::Alert;
use crate::config::Config;

#[derive(Serialize, Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub priority: isize,
}

impl Default for Notification {
    fn default() -> Notification {
        Notification {
            title: "".to_string(),
            message: "".to_string(),
            priority: 5,
        }
    }
}

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
