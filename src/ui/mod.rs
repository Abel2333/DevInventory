pub mod cli;
pub mod common;

pub use cli::run_cli;

use crate::config::Config;
use crate::crypto::MasterKey;
use anyhow::Result;

/// Display the result of initialization to the user
pub fn display_init_result(config: &Config, master_key: MasterKey) -> Result<()> {
    use base64::{Engine as _, engine::general_purpose};

    let key_base64 = general_purpose::STANDARD.encode(master_key.as_bytes());

    println!("✅ Database created at: {}", config.db_path.display());
    println!("\n✅ Master key generated:\n");
    println!("    {}\n", key_base64);
    println!("⚠️  IMPORTANT: Save this master key securely!");
    println!("    - Store in a password manager");
    println!("    - Write it down and keep in a safe place");
    println!("    You will need it to access your secrets.\n");

    if config.master_key_source.allow_keyring {
        println!("✅ Master key saved to system keyring");
        println!("   Service: {}", config.keyring_service);
        println!("   Account: {}", config.keyring_account);
    } else {
        println!("ℹ️  Keyring disabled. Use --dmk to provide key in future commands:");
        println!("   devinventory --dmk \"{}\" <command>", key_base64);
    }

    Ok(())
}
