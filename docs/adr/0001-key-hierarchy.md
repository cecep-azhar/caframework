# ADR 0001: Key Hierarchy and Zero-Knowledge Keyring (v1)

- **Status:** Accepted
- **Date:** 2026-10-04
- **Author:** CAFramework Build Agent / Cecep Azhar
- **Task:** F6 (Key hierarchy: DEK, BK, and BIP-39 Recovery Code)

## Context

Prior implementations of CAFramework used a direct Argon2id derivation from the master password into SQLCipher, without a level of indirection. This had major limitations:
1. **Password change required full database re-keying (`PRAGMA rekey`)**, which is slow and can fail mid-operation leaving the database in an inconsistent state.
2. **No account recovery capability**: If a user forgot their master password, all encrypted data was permanently lost.
3. **No independent backup encryption key**: Backups could not be encrypted with a dedicated key separate from the database runtime key.

## Decision

We adopt a two-tier key hierarchy modeled after modern zero-knowledge password managers:

```text
DEK  = 32 random bytes (OsRng)    -> SQLCipher raw database encryption key
BK   = 32 random bytes (OsRng)    -> Independent backup key (used in F11)

KEK_pw = Argon2id(master_password, salt_pw, m=64MiB, t=3, p=4)
KEK_rc = HKDF-SHA256(ikm = 256-bit recovery entropy, salt_rc, info = "caf.kek.recovery.v1")

keyring.v1.json (0600 permissions, atomic writes):
  {
    "format": 1,
    "vault_id": "<uuid-v4>",
    "kdf": { "alg": "argon2id", "m": 65536, "t": 3, "p": 4 },
    "password_slot": {
      "salt_pw": "<b64>",
      "nonce": "<b64-12B>",
      "ct": "<b64-80B (AES-256-GCM of DEK||BK with AAD)>"
    },
    "recovery_slot": {
      "salt_rc": "<b64>",
      "generation": 1,
      "nonce": "<b64-12B>",
      "ct": "<b64-80B (AES-256-GCM of DEK||BK with AAD)>"
    }
  }
```

### Key properties:
- **Instant Password Change**: Changing the master password only re-wraps `DEK||BK` in the `password_slot` using a newly derived `KEK_pw`. The database file itself is not rekeyed (`PRAGMA rekey` is completely eliminated).
- **Dual Wrapping**: Both `password_slot` and `recovery_slot` wrap the identical 64-byte payload (`DEK||BK`).
- **BIP-39 24-Word Recovery Code**: Setup generates 256 bits of cryptographically secure entropy formatted as a standard 24-word BIP-39 mnemonic. The plaintext words are never stored on disk.
- **Recovery Confirmation**: The user must confirm 3 randomly selected words before the vault transitions out of `RecoveryUnconfirmed`.
- **Revocable Recovery**: Regenerating the recovery code updates the recovery slot with new entropy and increments `generation`, instantly invalidating older paper backups.
- **Brute-Force Protection**: Password and recovery unlock attempts share a unified exponential backoff counter. After 5 consecutive failures, the vault enters `LockoutActive`.

## Consequences

- **Positive**:
  - Constant-time password change regardless of database size.
  - Safe, verified recovery workflow without compromising zero-knowledge guarantees.
  - No plaintext keys stored anywhere on the filesystem.
  - Property-based fuzz tests verify resilience against malformed/tampered keyring JSON.
- **Negative**:
  - Legacy vaults without `keyring.v1.json` are rejected with `VaultError::LegacyFormat` (as decided in D-5).
