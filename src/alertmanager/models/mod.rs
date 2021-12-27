use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alert {
    pub status: String,
    pub labels: Label,
    pub annotations: Annotation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Annotation {
    pub description: Option<String>,
    pub summary: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Label {
    pub alertname: String,
    pub job: String,
    pub instance: String,
}
