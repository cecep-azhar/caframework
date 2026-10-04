# Security Documentation — CAFramework v2

## 1. Dual-Level Lock Architecture
CAFramework implements a strictly enforced two-level locking hierarchy:
1. **Level 1 — Vault Encryption Lock (AES-256-GCM / Argon2id)**: Protects SQLite database at rest. Unlocked via Master Password or 24-word BIP-39 Recovery Code. Tested in `crates/caf-core/src/keyring.rs`.
2. **Level 2 — Profile Session & RBAC Lock**: Isolates multi-profile user data on shared machines. Verified via PIN or session capability matching. Tested in `crates/caf-core/src/session.rs` and `crates/caf-core/src/rbac.rs`.

## 2. Key Hierarchy & Zeroization
- **DEK (Database Encryption Key)**: 32-byte secret wrapping the database. Lives strictly in memory inside zeroizing buffers (`Zeroizing<[u8; 32]>`).
- **BK (Backup Key)**: 32-byte secret used for portable backup exports.
- **Recovery Slot**: Derived from BIP-39 24-word entropy.

Verified in `tests/arch.rs` and `crates/caf-core/tests/visibility_matrix.rs`.
