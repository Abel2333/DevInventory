mod config;
mod crypto;
mod crypto_service;
mod db;
mod domain;
mod keymgr;
mod service;
mod ui;

use anyhow::Result;
use clap::Parser;
use config::Config;
use crypto_service::CryptoService;
use db::Repository;
use env_logger::Env;
use keymgr::{MasterKeyProvider, MasterKeySource};
use log::info;
use service::SecretService;
use std::path::PathBuf;
use ui::cli::Commands;

/// Global arguments (can be used with any command)
#[derive(Parser)]
#[command(
    name = "devinventory",
    version,
    about = "Manage infrastructure secrets locally with encryption"
)]
struct Args {
    /// Database path override
    #[arg(long, global = true)]
    db_path: Option<PathBuf>,

    /// Master key (base64)
    #[arg(long, global = true)]
    dmk: Option<String>,

    /// Environment variable name for master key
    #[arg(long, global = true)]
    dmk_env: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // Initialize logger early; default to info level but allow RUST_LOG override.
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    info!("starting devinventory CLI");

    // 1. Parse arguments
    let args = Args::parse();

    // 2. Build configuration
    let master_key_source = MasterKeySource {
        base64_inline: args.dmk,
        env_name: None,
    };

    let config = Config::build(args.db_path, master_key_source, args.dmk_env)?;

    // 3. Handle Init command separately (it's a special initialization operation)
    if matches!(args.command, Commands::Init) {
        let key_provider = MasterKeyProvider::new(config.master_key_source.clone());
        let (_service, master_key) = SecretService::init(&config.db_path, &key_provider).await?;

        // Display the result to the user
        ui::display_init_result(&config, master_key)?;
        info!("devinventory initialized successfully");
        return Ok(());
    }

    // 4. Normal command flow: initialize infrastructure
    let repo = Repository::connect(&config.db_path).await?;
    repo.migrate().await?;

    let key_provider = MasterKeyProvider::new(config.master_key_source.clone());
    let key_result = key_provider.obtain(false)?;
    let crypto_service = CryptoService::new(key_result.into_key());
    let service = SecretService::new(repo, crypto_service);

    // 5. Run CLI
    ui::run_cli(
        service,
        args.command,
        config.master_key_source.env_name.clone(),
    )
    .await?;

    info!("devinventory CLI completed successfully");
    Ok(())
}
