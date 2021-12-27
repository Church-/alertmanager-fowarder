use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    alertmanager_forwarder::logger::init_logger()?;
    alertmanager_forwarder::rocket().await?.launch().await?;
    Ok(())
}
