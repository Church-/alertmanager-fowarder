use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub backtrace: Option<BacktraceConfig>,
    pub gotify: Gotify,
    pub port: Option<u16>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BacktraceConfig {
    pub token: String,
    pub url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Gotify {
    pub uri: String,
    pub token: String,
}
