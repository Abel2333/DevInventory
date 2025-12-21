use crate::{
    crypto_service::CryptoService,
    keymgr::{MasterKeyProvider, MasterKeySource},
    service::SecretService,
    ui::common::{SecretRow, mask},
};
use anyhow::Result;
use clap::Subcommand;
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

pub async fn run_cli(service: SecretService, command: Commands) -> Result<()> {
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
            let secret = service.get_secret(&name).await?;

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

            let rows: Vec<SecretRow> = metadata_list
                .into_iter()
                .map(|m| SecretRow {
                    name: m.name,
                    kind: m.kind.unwrap_or_default(),
                    created_at: m.created_at.to_rfc3339(),
                    updated_at: m.updated_at.to_rfc3339(),
                })
                .collect();

            let count = rows.len();
            let mut table = Table::new(rows);
            table.with(Style::rounded());

            info!("listed {} secrets (metadata only)", count);
            println!("{}", table);
        }

        Commands::Search { query } => {
            let metadata_list = service.search_secrets(&query).await?;

            let rows: Vec<SecretRow> = metadata_list
                .into_iter()
                .map(|m| SecretRow {
                    name: m.name,
                    kind: m.kind.unwrap_or_default(),
                    created_at: m.created_at.to_rfc3339(),
                    updated_at: m.updated_at.to_rfc3339(),
                })
                .collect();

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

            // 1. 创建新的密钥提供者（生成新密钥）
            let new_key_provider = MasterKeyProvider::new(MasterKeySource {
                base64_inline: None,
                allow_keyring: true,
            });

            // 2. 创建新的 CryptoService（generate_new = true）
            let new_crypto_service = CryptoService::new(&new_key_provider, true).await?;

            // 3. 执行密钥轮换
            service.rotate_master_key(new_crypto_service).await?;

            println!("✅ Master key rotated successfully!");
            println!("⚠️  New master key has been saved to your keyring");
            println!("    If keyring is not available, please save the key printed above");
        }
    }

    Ok(())
}
