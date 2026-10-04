# PRD Amendments & Deviations

This file records intentional amendments and deviations from `prd.md` v1.1.

## D-1: Window Section in `app.toml`
- **Context:** PRD FR-1 schema originally lacked an explicit `[window]` configuration block.
- **Decision:** `[window]` is supported as an optional section (width, height, min_width, min_height, resizable) to allow app builders to configure desktop window constraints.

## D-4: Pro Teams & Sync Deferral (v0.1.0 Scope)
- **Context:** PRD §4 keep list included full team licensing and cloud sync endpoints.
- **Decision:** For v0.1.0, Pro scope is restricted to client licensing validation (status, login, logout, register, trial, password reset, account, revoke device). ProTeamPanel and 9 team/sync endpoints are removed until the sync engine lands in future versions.

## D-5: No Legacy Vault Migration
- **Context:** Pre-release builds created vaults with `vault.key` plaintext fallback or Argon2(master) direct key derivation.
- **Decision:** No migration runner for legacy pre-release formats. The core returns `VaultError::LegacyFormat` on startup, instructing the user to clean the dev data directory.
