pub mod cli;

pub use cli::run_cli;

use crate::app::InitResult;
use anyhow::Result;

/// Display the result of initialization to the user
pub fn render_init_result(result: &InitResult) -> Result<()> {
    use base64::{Engine as _, engine::general_purpose};

    let key_base64 = general_purpose::STANDARD.encode(result.master_key.as_bytes());

    println!("✅ Database created at: {}", result.db_path.display());
    println!("\n✅ Master key generated:\n");
    println!("    {}\n", key_base64);
    println!("⚠️  IMPORTANT: Save this master key securely!");
    println!("    - Store in a password manager");
    println!("    - Write it down and keep in a safe place");
    println!("    You will need it to access your secrets.\n");

    println!("ℹ️  Use --dmk or set an environment variable for future commands:");
    println!("   devinventory --dmk \"{}\" <command>", key_base64);
    println!("   export DEVINVENTORY_DMK=\"{}\"", key_base64);

    Ok(())
}
