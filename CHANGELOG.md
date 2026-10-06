# Changelog

All notable changes to CAFramework will be documented in this file.

## [0.1.0-dev] - 2026-10-04
### Added
- Multi-profile architecture with RBAC permissions and PIN unlock.
- SQLCipher encryption with Argon2id DEK key-wrapping & BIP-39 recovery phrases.
- Svelte 5 runes UI with dark/light theming, titlebar controls, and i18n dictionaries.
- Generic AI context scrubber and assistant panel.
- Generic sync table structures with UUIDv7 IDs, monotonic revs, and change_log audit triggers.
- `caf-xtask` developer tooling (`codegen`, `guard`, `new-app`).

### Architecture
- Decoupled terminal and networking modules into clean, generic modular slices.
- Rewrote vault initialization with secure dual-slot key-wrapping.
- Ported frontend stores to Svelte 5 Runes.
