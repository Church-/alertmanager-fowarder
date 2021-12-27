use anyhow::Result;
use log::{info, warn};
use once_cell::sync::OnceCell;
use rocket::{fairing::AdHoc, http::Status};
use std::io::Cursor;
use tracing_subscriber::{
    filter::EnvFilter, layer::SubscriberExt, reload, util::SubscriberInitExt, Registry,
};

static LOG_RELOAD_HANDLE: OnceCell<LoggerReloadHandle> = OnceCell::new();

type LoggerReloadHandle = reload::Handle<tracing_subscriber::filter::EnvFilter, Registry>;

fn install_logger() -> Result<LoggerReloadHandle> {
    const DEFAULT_CONFIG: &str = "info,rocket=warn,launch=warn,_=warn,launch_=warn";

    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(DEFAULT_CONFIG.to_string()))?;
    let (filter, reload) = reload::Layer::new(env_filter);
    let registry = Registry::default().with(filter);

    // Skip trying to log to journald if this is running in a tty.
    // This conditional intends to cover the fact that interactive shells are frequently run within
    // a systemd container and thus can technically "log" to journald.
    let try_journald = !atty::is(atty::Stream::Stderr);

    if try_journald {
        match tracing_journald::layer() {
            Ok(writer) => {
                registry.with(writer).try_init()?;
                eprintln!("Tracing to journald.");
                return Ok(reload);
            }
            Err(err) => eprintln!(
                "failed to connect to journald; defaulting to stderr. err={:?}",
                err
            ),
        }
    }

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_ids(true)
        .compact()
        .with_writer(std::io::stderr);
    registry.with(fmt_layer).try_init()?;
    eprintln!("Tracing to stderr.");

    Ok(reload)
}

pub fn init_logger() -> Result<LoggerReloadHandle> {
    LOG_RELOAD_HANDLE
        .get_or_try_init(|| install_logger())
        .map(|handle| handle.clone())
}

pub fn log_requests() -> AdHoc {
    AdHoc::on_response("Request Logger", |req, resp| {
        Box::pin(async move {
            let status = resp.status();

            let ip = match req.headers().get("X-Proxied-For").next() {
                Some(a) => a.to_string(),
                None => req
                    .client_ip()
                    .map_or_else(|| "-".into(), |a| a.to_string()),
            };
            let method = req.method().as_str();
            let uri = req.uri().clone().into_normalized();
            let uri_qs = uri
                .query()
                .map(|q| format!("?{}", q))
                .unwrap_or_else(|| "".into());
            let uri_str = format!("{}{}", uri.path(), uri_qs);

            if status == Status::Ok {
                info!("{} {} {} {}", ip, method, uri_str, status.code);
                return;
            }

            resp.body_mut().to_string().await.map_or_else(
                |_| warn!("{} {} {} {}", ip, method, uri_str, status.code),
                |bstr| {
                    // Log the response body, then restore it.
                    warn!("{} {} {} {}: {}", ip, method, uri_str, status.code, bstr);
                    resp.set_sized_body(bstr.len(), Cursor::new(bstr));
                },
            );
        })
    })
}
