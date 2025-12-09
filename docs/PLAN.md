# Secure Secrets Plan

## Goal
Design and implement secrets storage for DevInventory so service tokens/passwords are encrypted at rest, with a single master key disclosed only once and (optionally) stored in the OS keyring.

## Scope
- CLI-only MVP; TUI/GUI reuse same core.
- SQLite data store already in repo; add migrations and crypto wrapper.
- Support two modes: (a) keyring-backed (default), (b) user-managed `--no-keyring` (print-once).

## Milestones
1) **Schema & migrations**
   - Add migrations for secrets table/columns and audit table.
   - Add `created_at/updated_at`, unique indexes.
2) **Key management**
   - Implement DMK bootstrap: load from keyring → else generate 32B random.
   - Print-once flow with `--no-keyring`; warning + prompt to save.
   - Support `--dmk <base64>` override for headless use.
3) **Crypto layer**
   - Build `Secret` wrapper: AEAD (AES-256-GCM or ChaCha20-Poly1305), random nonce, AAD includes record id/type.
   - Zeroize buffers after use; no Debug impl prints data.
4) **CLI surface**
   - Commands: `secret add|get|list|rm`, `key rotate`, `backup`, `restore --dmk`.
   - Inputs via hidden prompt; outputs masked by default, plaintext only with `--show` + confirmation.
5) **Testing**
   - Offline sqlx tests with temp SQLite; crypto round-trip tests; rotate rollback tests.
   - Fuzz/prop tests for serialize/deserialize.
6) **Docs & warnings**
   - Document backup steps: copy DB + save DMK; loss of DMK makes data unrecoverable.
   - Explain modes and threat model; emphasize least-privilege file perms (600).

## Open Questions
- Need SQLCipher full-DB encryption, or field-level enough?
- Multi-user/team sharing: move to server/KMS later?
- Token TTL/rotation policies required?

## Timeline (rough)
- Milestones 1–3: 1–2 days.
- Milestones 4–5: 1 day.
- Docs & polish: 0.5 day.
