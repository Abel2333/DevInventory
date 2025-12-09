# DevInventory Usage

## Features Implemented
- Local SQLite datastore with automatic table creation and indexing.
- Field-level encryption using ChaCha20-Poly1305; AAD binds to secret name.
- Master key (32B) bootstrap: inline `--dmk`, keyring lookup, or generate+print once (optional `--no-keyring`).
- Secret commands: add/get/list/rm; masked output by default, `--show` to reveal.
- Key rotation re-encrypts all secrets and updates keyring when allowed.

## Default Paths
- DB: `~/.config/devinventory/devinventory.db` (override with `--db-path`).
- Keyring entry: service `devinventory`, account `dmk` (skipped if `--no-keyring`).

## Common Commands
- Add (prompted secret): `devinventory add api-token --kind token --note "prod"`
- Add (inline value): `devinventory add db-pass --value 'P@ssw0rd'`
- Get masked: `devinventory get api-token`
- Get plaintext: `devinventory get api-token --show`
- List metadata: `devinventory list`
- Remove: `devinventory rm api-token`
- Rotate master key: `devinventory rotate`
- Use custom DB path: `devinventory --db-path ./secrets.db list`
- Headless DMK: `devinventory --dmk BASE64KEY add ...`

## Safety Defaults
- Secrets never printed unless `--show`.
- Inputs without `--value` use no-echo prompt.
- `.gitignore` excludes `*.db`.

## Tests
- Unit: crypto round-trip.
- Integration: SQLite CRUD + key rotation using in-memory DB.
- Run all: `cargo test`
