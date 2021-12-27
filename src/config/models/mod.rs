use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub gotify: Gotify,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Gotify {
    pub uri: String,
    pub token: String,
}
