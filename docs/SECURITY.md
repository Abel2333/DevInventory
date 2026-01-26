# Secrets Handling Guide

## Master Key (DMK)
- 32-byte random key; base64 is printed only when first created or on rotation.
- Provide explicitly with `--dmk <base64>` for headless/CI.
- Or set via environment variable (default name `DEVINVENTORY_DMK`); override with `--dmk-env` or `key.env_name` in config.
- Loss of the DMK means existing secrets cannot be decrypted.

## Encryption
- Field-level encryption using ChaCha20-Poly1305 (AEAD) with random 96-bit nonce; AAD includes the secret name.
- Ciphertext stored in SQLite; DB backups are safe to sync without the DMK.

## Key Rotation
- `devinventory rotate` generates a new DMK, re-encrypts all secrets, and prints the new key once.

## Input/Output Hygiene
- Secret input uses no-echo prompt when `--value` is omitted.
- `get` masks output by default; plaintext requires `--show`.
- Logs never contain plaintext secrets.

## Files & Permissions
- Default DB path: `~/.config/devinventory/devinventory.db` (override with `--db-path`).
- Ensure the config directory and DB file are mode 600 when possible.
- `.gitignore` excludes `*.db` to prevent accidental commits.

## Backup & Restore
- Backup = copy the DB file **plus** store the DMK (base64) in a secure vault.
- Restore on a new machine: place DB file, then set DMK via env or `--dmk`, then run commands as usual.
