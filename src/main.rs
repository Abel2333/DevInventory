mod cli;
mod crypto;
mod db;
mod keymgr;

use anyhow::Result;
use env_logger::Env;
use log::info;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // Initialize logger early; default to info level but allow RUST_LOG override.
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    info!("starting devinventory CLI");
    cli::run().await
}
