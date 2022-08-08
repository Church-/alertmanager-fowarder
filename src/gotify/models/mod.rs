use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub priority: String,
}

impl Default for Notification {
    fn default() -> Notification {
        Notification {
            title: "".to_string(),
            message: "".to_string(),
            priority: 5.to_string(),
        }
    }
}
