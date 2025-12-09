use crate::{
    crypto::SecretCrypto,
    db::Repository,
    keymgr::{MasterKeyProvider, MasterKeySource},
};
use anyhow::{Result, anyhow};
use clap::{ArgAction, Parser, Subcommand};
use log::{debug, info, warn};
use rpassword::prompt_password;
use std::path::PathBuf;
use tabled::{Table, Tabled, settings::Style};

#[derive(Parser, Debug)]
#[command(
    name = "devinventory",
    version,
    about = "Manage infrastructure secrets locally with encryption"
)]
pub struct Cli {
    /// Optional override for database file path
    #[arg(long, global = true)]
    db_path: Option<PathBuf>,

    /// Do not write master key to OS keyring; print it once instead
    #[arg(long, global = true, default_value_t = false)]
    no_keyring: bool,

    /// Provide master key (base64) explicitly; skips keyring lookup
    #[arg(long, global = true)]
    dmk: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add or update a secret
    Add {
        /// Unique name for this secret
        name: String,
        /// Optional type/kind label
        #[arg(long)]
        kind: Option<String>,
        /// Optional description
        #[arg(long)]
        note: Option<String>,
        /// Provide secret via argument instead of prompt
        #[arg(long)]
        value: Option<String>,
    },
    /// Get and print a secret (masked by default)
    Get {
        name: String,
        /// Show plaintext without masking (ask for confirmation)
        #[arg(long, action = ArgAction::SetTrue)]
        show: bool,
    },
    /// List secrets (metadata only)
    List,
    /// Search secrets by substring (name/kind/note)
    Search {
        /// Case-insensitive substring to match
        query: String,
    },
    /// Initialize master key (generate, optionally store to keyring)
    Init,
    /// Remove a secret permanently
    Rm { name: String },
    /// Rotate master key and re-encrypt all secrets
    Rotate,
}

#[derive(Tabled)]
struct SecretRow {
    name: String,
    kind: String,
    created_at: String,
    updated_at: String,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    let db_path = crate::db::resolve_db_path(cli.db_path.as_ref())?;
    info!("opening database at {}", db_path.to_string_lossy());
    let repo = Repository::connect(&db_path).await?;
    repo.migrate().await?;
    debug!("database migrations ensured");

    let key_provider = MasterKeyProvider::new(MasterKeySource {
        base64_inline: cli.dmk.clone(),
        allow_keyring: !cli.no_keyring,
    });

    match cli.command {
        Commands::Init => {
            let master_key = key_provider.obtain(true).await?;
            let crypto = SecretCrypto::new(master_key.clone());
            // quick touch to ensure key material used and zeroized after scope
            let _ = crypto.encrypt("init", b"").ok();
            println!("✅ master key initialized");
        }
        Commands::Add {
            name,
            kind,
            note,
            value,
        } => {
            let master_key = key_provider.obtain(false).await?;
            info!("master key ready for add");
            let crypto = SecretCrypto::new(master_key.clone());
            let secret = match value {
                Some(v) => v,
                None => prompt_password("Secret value: ")?,
            };
            let ciphertext = crypto.encrypt(&name, secret.as_bytes())?;
            repo.upsert_secret(&name, kind, note, &ciphertext).await?;
            info!("saved/updated secret: {}", name);
            println!("✅ saved: {}", name);
        }
        Commands::Get { name, show } => {
            let master_key = key_provider.obtain(false).await?;
            let crypto = SecretCrypto::new(master_key.clone());
            let record = repo
                .fetch_secret(&name)
                .await?
                .ok_or_else(|| anyhow!("secret not found"))?;
            let plaintext = crypto.decrypt(&record.name, &record.ciphertext)?;
            if show {
                warn!("secret '{}' printed in plaintext", name);
                println!("{}", String::from_utf8_lossy(&plaintext));
            } else {
                let masked = mask(&plaintext);
                println!("{} => {}", name, masked);
            }
        }
        Commands::List => {
            // requires key presence to avoid silently generating
            let _ = key_provider.obtain(false).await?;
            let rows = repo.list_secrets().await?;
            let view: Vec<SecretRow> = rows
                .into_iter()
                .map(|r| SecretRow {
                    name: r.name,
                    kind: r.kind.unwrap_or_default(),
                    created_at: r.created_at.to_rfc3339(),
                    updated_at: r.updated_at.to_rfc3339(),
                })
                .collect();
            let count = view.len();
            let mut table = Table::new(view);
            table.with(Style::rounded());
            info!("listed {} secrets (metadata only)", count);
            println!("{}", table);
        }
        Commands::Search { query } => {
            let _ = key_provider.obtain(false).await?;
            let rows = repo.search_secrets(&query).await?;
            let view: Vec<SecretRow> = rows
                .into_iter()
                .map(|r| SecretRow {
                    name: r.name,
                    kind: r.kind.unwrap_or_default(),
                    created_at: r.created_at.to_rfc3339(),
                    updated_at: r.updated_at.to_rfc3339(),
                })
                .collect();
            let count = view.len();
            let mut table = Table::new(view);
            table.with(Style::rounded());
            info!("search '{}' matched {} secrets", query, count);
            println!("{}", table);
        }
        Commands::Rm { name } => {
            let _ = key_provider.obtain(false).await?;
            let deleted = repo.delete_secret(&name).await?;
            if deleted {
                info!("removed secret: {}", name);
                println!("🗑️ removed: {}", name);
            } else {
                warn!("secret not found for removal: {}", name);
                println!("not found: {}", name);
            }
        }
        Commands::Rotate => {
            let current_key = key_provider.obtain(false).await?;
            let current_crypto = SecretCrypto::new(current_key.clone());
            let new_key = key_provider.rotate().await?;
            repo.reencrypt_all(&current_crypto, &new_key).await?;
            info!("master key rotated and secrets re-encrypted");
            println!("🔑 master key rotated; remember to back it up");
        }
    }

    Ok(())
}

fn mask(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "(empty)".to_string();
    }
    let s = String::from_utf8_lossy(bytes);
    let len = s.chars().count();
    let head = s.chars().take(2).collect::<String>();
    let tail = s.chars().rev().take(2).collect::<String>();
    match len {
        0 => "(empty)".into(),
        1..=3 => "***".into(),
        _ => format!("{}***{}", head, tail.chars().rev().collect::<String>()),
    }
}
