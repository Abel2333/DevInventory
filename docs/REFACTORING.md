# DevInventory Architecture Refactor Plan

## Goal

Refactor the project to reduce coupling and prepare for a future TUI.

## Current Architecture Issues

### Major Issues

1. **Business logic embedded in CLI** - `cli.rs` mixes command parsing, user interaction, formatting, and orchestration.
2. **Missing service/domain layer** - CLI calls db and crypto directly with no abstraction.
3. **Presentation mixed with business logic** - `mask()`, `SecretRow`, and `println!` calls are scattered through core logic.
4. **Incomplete repository boundary** - `reencrypt_all()` depends directly on `SecretCrypto`.
5. **Scattered config handling** - `resolve_db_path()` lives in `db.rs`, CLI args are passed around directly.

### Current Structure (~720 LOC)

```
main.rs (19 lines)   - entry point
cli.rs (212 lines)   - CLI + all business logic + UI
Db.rs (270 lines)    - repository + migrations
crypto.rs (88 lines) - crypto primitives (well isolated)
keymgr.rs (131 lines)- master key lifecycle
```

## Target Architecture

```
src/
  main.rs           - minimal entry point
  config.rs         - config management (new)
  domain.rs         - domain models (new)
  service.rs        - service/business layer (new)
  crypto.rs         - crypto primitives (unchanged)
  db.rs             - repository (light changes)
  keymgr.rs         - key management (light changes)
  ui/
    mod.rs          - UI module exports (new)
    cli.rs          - CLI UI (Clap + formatting) (new)
    common.rs       - shared UI utilities (mask, etc.) (new)
    (tui.rs)        - future TUI placeholder
```

### Layering

```
┌─────────────────────────────────────────┐
│         UI layer (ui/)                  │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  │
│  │ cli.rs  │  │ tui.rs  │  │ common  │  │
│  └─────────┘  └─────────┘  └─────────┘  │
└──────────────┬──────────────────────────┘
               │ calls
┌──────────────▼──────────────────────────┐
│       Service layer (service.rs)        │
│     SecretService - core business       │
└──────────────┬──────────────────────────┘
               │ uses
┌──────────────▼──────────────────────────┐
│     Infrastructure layer                │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐   │
│  │  db.rs  │ │keymgr.rs│ │crypto.rs │   │
│  └─────────┘ └─────────┘ └──────────┘   │
└─────────────────────────────────────────┘
          ▲
          │ uses
┌─────────┴───────────┐
│   domain.rs         │
│   (domain models)   │
└─────────────────────┘
```

### Data Flow Example (Add Secret)

```
User runs "devinventory add github-token"
  │
  ▼
ui/cli.rs - parse args, prompt input
  │ calls: service.add_secret("github-token", value, ...)
  ▼
service.rs - SecretService::add_secret()
  │ - obtain master key
  │ - encrypt data
  │ - persist to DB
  │ returns: Result<Secret>
  ▼
ui/cli.rs - format output "✓ Secret added"
```

## Refactor Steps

### Phase 1: Create foundation modules (non-breaking)

**1.1 Create `src/domain.rs`**
Define domain models separate from DB and UI:

```rust
pub struct Secret {
    pub id: Uuid,
    pub name: String,
    pub kind: Option<String>,
    pub note: Option<String>,
    pub plaintext: Vec<u8>,  // in-memory only
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct SecretMetadata {
    pub id: Uuid,
    pub name: String,
    pub kind: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**1.2 Create `src/config.rs`**
Centralized config management:

```rust
pub struct Config {
    pub db_path: PathBuf,
    pub master_key_source: MasterKeySource,
}

impl Config {
    pub fn from_env() -> Result<Self> { ... }
    pub fn resolve_db_path() -> Result<PathBuf> { ... }
}
```

- Move `resolve_db_path()` out of `db.rs`
- Convert CLI args into a config object

**1.3 Create `src/service.rs`**
Core business logic layer:

```rust
pub struct SecretService {
    repo: Repository,
    key_provider: MasterKeyProvider,
}

impl SecretService {
    pub fn new(repo: Repository, key_provider: MasterKeyProvider) -> Self

    pub async fn add_secret(
        &self,
        name: String,
        value: Vec<u8>,
        kind: Option<String>,
        note: Option<String>,
    ) -> Result<Secret>

    pub async fn get_secret(&self, name: &str) -> Result<Secret>

    pub async fn list_secrets(&self) -> Result<Vec<SecretMetadata>>

    pub async fn search_secrets(&self, query: &str) -> Result<Vec<SecretMetadata>>

    pub async fn delete_secret(&self, name: &str) -> Result<()>

    pub async fn rotate_master_key(&self) -> Result<()>
}
```

- Encapsulate crypto + keymgr + db coordination
- Return domain models instead of DB records

### Phase 2: Create UI module

**2.1 Create `src/ui/mod.rs`**

```rust
pub mod cli;
pub mod common;

pub use cli::run_cli;
```

**2.2 Create `src/ui/common.rs`**
Move presentation helpers from `cli.rs`:

```rust
pub fn mask(s: &str) -> String { ... }

pub struct SecretDisplayRow {
    // for tabled output
}
```

**2.3 Create `src/ui/cli.rs`**
CLI concerns only:

- Clap command definitions
- User interaction (rpassword prompts)
- Formatting output (tabled)
- Call `SecretService`
- **No** crypto/db/key management logic

### Phase 3: Migrate business logic

**3.1 Move command handlers from `cli.rs` to `service.rs`**

Current (cli.rs):

```rust
Commands::Add { name, value, kind, note } => {
    let key = key_provider.obtain(false)?;
    let crypto = SecretCrypto::new(key);
    let plaintext = /* prompt or use value */;
    let ciphertext = crypto.encrypt(...)?;
    repo.upsert_secret(...)?;
    println!("✓ Secret added");
}
```

Refactor to:

```rust
// UI (ui/cli.rs)
Commands::Add { name, value, kind, note } => {
    let plaintext = /* prompt or use value */;
    let secret = service.add_secret(name, plaintext, kind, note).await?;
    println!("✓ Secret '{}' added", secret.name);
}

// Service (service.rs)
pub async fn add_secret(...) -> Result<Secret> {
    let key = self.key_provider.obtain(false)?;
    let crypto = SecretCrypto::new(key);
    let ciphertext = crypto.encrypt(...)?;
    let record = self.repo.upsert_secret(...).await?;
    Ok(Secret { ... })
}
```

**3.2 Decouple `reencrypt_all` in `db.rs`**

Current problem:

```rust
pub async fn reencrypt_all(&self, old: &SecretCrypto, new: &SecretCrypto) -> Result<()>
```

Option A (function pointers):

```rust
pub async fn reencrypt_all<F>(
    &self,
    decrypt_fn: F,
    encrypt_fn: F,
) -> Result<()>
where
    F: Fn(&[u8]) -> Result<Vec<u8>>
```

Option B (service layer):

```rust
pub async fn rotate_master_key(&self) -> Result<()> {
    let old_key = /* existing key */;
    let new_key = /* new key */;
    let old_crypto = SecretCrypto::new(old_key);
    let new_crypto = SecretCrypto::new(new_key);

    let records = self.repo.list_secrets(None).await?;
    for record in records {
        let plaintext = old_crypto.decrypt(&record.ciphertext, ...)?;
        let new_ciphertext = new_crypto.encrypt(&plaintext, ...)?;
        self.repo.update_ciphertext(record.id, new_ciphertext).await?;
    }

    Ok(())
}
```

### Phase 4: Update entry point

**4.1 Simplify `main.rs`**

```rust
mod config;
mod crypto;
mod db;
mod domain;
mod keymgr;
mod service;
mod ui;

use anyhow::Result;
use config::Config;
use db::Repository;
use keymgr::MasterKeyProvider;
use service::SecretService;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = Config::from_env()?;

    let repo = Repository::connect(&config.db_path).await?;
    repo.migrate().await?;

    let key_provider = MasterKeyProvider::new(config.master_key_source);

    let service = SecretService::new(repo, key_provider);

    ui::cli::run_cli(service).await?;

    Ok(())
}
```

### Phase 5: Cleanup and testing

**5.1 Remove old `cli.rs`**

- All logic moved to `ui/cli.rs` and `service.rs`

**5.2 Add service-layer tests**

`service.rs` is now testable:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_secret() {
        let repo = Repository::connect(":memory:").await.unwrap();
        // ...
    }
}
```

**5.3 Update `Cargo.toml` if needed**

Check for new dependencies or feature flags.

## Key Decisions and Trade-offs

### Decision 1: SecretRecord vs Secret

**Decision**: Keep `SecretRecord` in `db.rs`, introduce `Secret` in `domain.rs`
**Rationale**:

- `SecretRecord` includes `ciphertext` (DB representation)
- `Secret` includes `plaintext` (domain representation)
- Repository converts `SecretRecord` ↔ `Secret`

### Decision 2: Service placement

**Decision**: Use a single `service.rs` instead of a `services/` directory
**Rationale**:

- Only one service (SecretService)
- Keep it simple for a small project
- Easy to split later if needed

### Decision 3: Error handling

**Decision**: Keep using `anyhow::Result`, no custom error types
**Rationale**:

- Sufficient for a CLI tool
- Avoid over-engineering
- Can migrate later if needed

### Decision 4: Async boundary

**Decision**: Keep service layer async (repository is async)
**Rationale**:

- Repository uses async SQLite
- Service layer naturally async
- UI layer can handle async as needed

### Decision 5: Dependency injection vs globals

**Decision**: Inject dependencies via constructors
**Rationale**:

- Easier to test
- Clearer dependencies
- Aligns with Rust best practices

### Decision 6: Master key sources

**Decision**: Only support CLI `--dmk` and environment variables (default `DEVINVENTORY_DMK`), no OS keyring
**Rationale**:

- Simplifies deployment and testing
- Avoids platform-specific keyring issues
- UI layer owns user prompts to save the key

## TUI Readiness

After refactor, adding a TUI becomes straightforward:

```rust
// src/ui/tui.rs (future)
// Use ratatui or cursive
// Call the same service methods
// No duplicate business logic
```

```rust
// ... initialize service ...
// Select UI based on CLI args
```

## Key File Change List

### New Files

- `src/config.rs` - config management
- `src/domain.rs` - domain models
- `src/service.rs` - business logic service layer
- `src/ui/mod.rs` - UI module exports
- `src/ui/cli.rs` - CLI UI
- `src/ui/common.rs` - shared UI helpers

### Modified Files

- `src/main.rs` - simplified to config + init + UI
- `src/db.rs` - remove `resolve_db_path()`, optionally refactor `reencrypt_all()`

### Removed Files

- `src/cli.rs` - split into `ui/cli.rs` and `service.rs`

## Verification Checklist

- [ ] All original commands still work (init, add, get, list, search, rm, rotate)
- [ ] Existing tests pass (crypto and db tests)
- [ ] Service layer has new unit tests
- [ ] LOC roughly similar (~750)
- [ ] `ui/cli.rs` contains only UI concerns, no crypto/db logic
- [ ] Easy to mock a basic TUI entry point (even if not implemented)

## Expected Benefits

1. **Lower coupling**: UI, business logic, and infrastructure are clearly separated
2. **Testability**: service layer can be tested in isolation
3. **Extensibility**: adding a TUI only needs a new `ui/tui.rs`
4. **Maintainability**: each module has a single responsibility
5. **Type safety**: domain vs DB models are separated to reduce confusion
