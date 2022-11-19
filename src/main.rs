use std::{borrow::Cow, collections::HashMap, net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use axum::{
    async_trait,
    extract::{rejection::JsonRejection, Extension, FromRequest, RequestParts},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    BoxError, Router,
};
use backtraceio::ResultExt;
use clap::Parser;
use gethostname::gethostname;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tracing::info;

use alertmanager::Payload;
use config::Config;
use gotify::{forward_notification, gen_notification};

mod alertmanager;
mod args;
mod config;
mod gotify;

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .compact()
        .with_writer(std::io::stderr)
        .try_init()
        .err()
        .map(|e| format!("{:?}", e));

    let args = args::Args::parse();
    let config_str = std::fs::read_to_string(args.config)?;
    let config: Config = toml::from_str(&config_str)?;

    if let Some(backtrace) = &config.backtrace {
        let mut attributes: HashMap<String, String> = HashMap::new();
        attributes.insert(
            String::from("application"),
            String::from("alertmanager-forwarder"),
        );

        if let Ok(hostname) = gethostname().into_string() {
            attributes.insert(String::from("hostname"), hostname);
        }

        backtraceio::init(
            &backtrace.token,
            &backtrace.url,
            None,
            Some(attributes.clone()),
        );
        backtraceio::register_error_handler(
            &backtrace.url,
            &backtrace.token,
            move |r: &mut backtraceio::Report, _| {
                for (key, value) in &attributes.clone() {
                    r.attributes.insert(key.to_string(), value.to_string());
                }
            },
        );
    }

    let shared_state = Arc::new(config.clone());

    let app = Router::new()
        .route("/forward_to_gotify", post(route_to_gotify))
        .layer(Extension(shared_state));

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port.unwrap_or(6064)));

    info!("Starting Alert Manager forwarding service...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn route_to_gotify(
    Json(payload): Json<Payload>,
    Extension(config): Extension<Arc<Config>>,
) -> impl IntoResponse {
    for alert in payload.alerts {
        let notification = gen_notification(alert);
        _ = forward_notification(notification, &config)
            .await
            .context("Failed to forward alert to gotify")
            .submit_error();
    }
}

// We define our own `Json` extractor that customizes the error from `axum::Json`
struct Json<T>(T);

#[async_trait]
impl<B, T> FromRequest<B> for Json<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::Json`
    T: DeserializeOwned,
    B: axum::body::HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = (StatusCode, axum::Json<Value>);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req).await.submit_error() {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                // convert the error from `axum::Json` into whatever we want
                let (status, body): (_, Cow<'_, str>) = match rejection {
                    JsonRejection::JsonDataError(err) => (
                        StatusCode::BAD_REQUEST,
                        format!("Invalid JSON request: {}", err).into(),
                    ),
                    JsonRejection::MissingJsonContentType(err) => {
                        (StatusCode::BAD_REQUEST, err.to_string().into())
                    }
                    err => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unknown internal error: {}", err).into(),
                    ),
                };

                Err((
                    status,
                    // we use `axum::Json` here to generate a JSON response
                    // body but you can use whatever response you want
                    axum::Json(json!({ "error": body })),
                ))
            }
        }
    }
}
