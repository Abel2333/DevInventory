use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use devinventory::{
    config::Config,
    crypto::CryptoService,
    keymgr::{MasterKeyProvider, MasterKeySource},
    storage::Repository,
    ui::tui::run_tui,
};
use env_logger::Env;

/// Global arguments for the TUI launcher.
#[derive(Parser)]
#[command(
    name = "devinventory-tui",
    version,
    about = "Launch the DevInventory terminal UI"
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
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let args = Args::parse();

    let master_key_source = MasterKeySource {
        base64_inline: args.dmk,
        env_name: None,
    };
    let config = Config::build(args.db_path, master_key_source, args.dmk_env)?;

    let repo = Repository::connect(&config.db_path).await?;
    repo.migrate().await?;

    let key_provider = MasterKeyProvider::new(config.master_key_source.clone());
    let key_result = key_provider.obtain(false)?;
    let crypto_service = CryptoService::new(key_result.into_key());
    let service = devinventory::app::SecretService::new(repo, crypto_service);

    run_tui(service).await
}
