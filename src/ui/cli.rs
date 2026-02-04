mod common;

use crate::{
    app::SecretService,
    crypto::CryptoService,
    keymgr::{MasterKeyProvider, MasterKeySource},
};
use anyhow::Result;
use clap::Subcommand;
use common::{SecretRow, mask};
use log::{info, warn};
use rpassword::prompt_password;
use tabled::{Table, settings::Style};

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Initialize master key
    Init,

    /// Add or update a secret
    Add {
        name: String,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        note: Option<String>,
        #[arg(long)]
        value: Option<String>,
    },

    /// Get a secret
    Get {
        name: String,
        #[arg(long)]
        show: bool,
    },

    /// List all secrets
    List,

    /// Search secrets
    Search { query: String },

    /// Remove a secret
    Rm { name: String },

    /// Rotate master key
    Rotate,
}

pub async fn run_cli(
    service: SecretService,
    command: Commands,
    env_name: Option<String>,
) -> Result<()> {
    match command {
        Commands::Init => {
            unreachable!("Init command should be handled in main before service creation")
        }

        Commands::Add {
            name,
            kind,
            note,
            value,
        } => {
            let secret_value = match value {
                Some(v) => v,
                None => prompt_password("Secret value: ")?,
            };

            let result = service
                .add_secret(name, secret_value.as_bytes().to_vec(), kind, note)
                .await?;

            info!("saved/updated secret: {}", result.name);
            println!("✅ saved: {}", result.name);
        }

        Commands::Get { name, show } => {
            let secret = service.get_secret_by_name(&name).await?;

            if show {
                warn!("secret '{}' printed in plaintext", name);
                println!("{}", String::from_utf8_lossy(&secret.plaintext));
            } else {
                let masked = mask(&secret.plaintext);
                println!("{} => {}", name, masked);
            }
        }

        Commands::List => {
            let metadata_list = service.list_secrets().await?;

            let rows = SecretRow::from_metadata_list(metadata_list);

            let count = rows.len();
            let mut table = Table::new(rows);
            table.with(Style::rounded());

            info!("listed {} secrets (metadata only)", count);
            println!("{}", table);
        }

        Commands::Search { query } => {
            let metadata_list = service.search_secrets(&query).await?;

            let rows = SecretRow::from_metadata_list(metadata_list);

            info!("search_secrets '{}' -> {} rows", query, rows.len());

            let mut table = Table::new(rows);
            table.with(Style::rounded());
            println!("{}", table);
        }

        Commands::Rm { name } => {
            service.delete_secret(&name).await?;
            println!("✅ deleted: {}", name);
        }

        Commands::Rotate => {
            println!("⚠️  Rotating master key...");

            // 1. Create a new key provider (generate a new key)
            let new_key_provider = MasterKeyProvider::new(MasterKeySource {
                base64_inline: None,
                env_name,
            });

            // 2. Create a new CryptoService (generate new = true)
            let key_result = new_key_provider.rotate()?;

            println!(
                "Generate new Master Key successfully, new key: {}",
                key_result.key_b64()
            );

            let new_crypto_service = CryptoService::new(key_result.into_key());

            // 3. Perform key rotation
            service.rotate_master_key(new_crypto_service).await?;

            println!("✅ Master key rotated successfully!");
            println!("Please save the key printed above");
        }
    }

    Ok(())
}
