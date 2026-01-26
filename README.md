# DevInventory

DevInventory is a local secrets manager CLI. It stores encrypted secrets in a local SQLite database and is intended for personal or small-team use on developer machines.

## Features
- Local SQLite storage with unique secret names
- ChaCha20-Poly1305 encryption/decryption
- Master key via CLI or environment variable
- Init, add/get/list/search/remove, and master key rotation

## Quick Start
### Build
```bash
cargo build
```

### Initialize
```bash
cargo run -- init
```
The command prints a base64 master key. Store it securely.

### Usage Examples
```bash
# Add or update a secret
cargo run -- --dmk "<BASE64_KEY>" add api --kind token --note prod --value secret123

# List all secrets (metadata only)
cargo run -- --dmk "<BASE64_KEY>" list

# Get a secret (masked by default)
cargo run -- --dmk "<BASE64_KEY>" get api

# Show plaintext
cargo run -- --dmk "<BASE64_KEY>" get api --show

# Search
cargo run -- --dmk "<BASE64_KEY>" search prod

# Remove
cargo run -- --dmk "<BASE64_KEY>" rm api
```

## Configuration
Priority: CLI args > environment variables > config file > defaults

### Environment Variables
- `DEVINVENTORY_DB_PATH`: database path (optional)
- `DEVINVENTORY_DMK`: base64 master key

### Config File
Default path (Linux/macOS): `$XDG_CONFIG_HOME/devinventory/config.toml`  
Windows: `%APPDATA%/devinventory/config.toml`

Example:
```toml
[database]
path = "secrets.db"

[key]
env_name = "DEVINVENTORY_DMK"
```

## Commands
```text
init      Initialize database and generate master key
add       Add or update a secret
get       Retrieve a secret
list      List all secrets (metadata)
search    Search secrets
rm        Remove a secret
rotate    Rotate master key and re-encrypt all entries
```

## Tests
```bash
cargo test
```

## Security Notes
- Losing the master key means stored secrets are unrecoverable.
- Store the master key in a password manager or secure env injection.
