# ADR 0002: Backup Format v2

## Context
CAFramework v2 requires a portable, authenticated, encrypted backup format supporting both manual passphrase export and automated scheduled backups.

## Structure
```
magic "CAFBAK\0\0" (8 bytes) | format_version u16 = 2 | header_len u32 | header JSON | ciphertext
```
- Cipher: AES-256-GCM
- KDF: Argon2id with random 32-byte salt per backup file
- Atomicity: Restores occur inside a single SQLite transaction.

## Consequences
- Guaranteed forward compatibility
- Multi-profile isolation preserved on backup & restore.
