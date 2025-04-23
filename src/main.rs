use logger::init_logging;
use stats::parse_stats;

mod config;
mod fs;
mod logger;
mod stats;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init_logging();
    if let Err(e) = parse_stats().await {
        tracing::error!("Something went wrong when getting allocation: {e}")
    }
    Ok(())
}
