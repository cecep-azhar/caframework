# CAFramework System Architecture

## 1. High-Level Architecture
CAFramework is an open-source, modular enterprise starter framework for cross-platform desktop (Linux, Windows, macOS) and mobile (Android) apps.

```
┌─────────────────────────────────────────────────────────────┐
│                 Frontend (Svelte 5 + Runes)                 │
│   - Reactivity with `$state`, `$derived`, `$props`          │
│   - Tailwind CSS v4 + CADS Design Tokens (Crystalline Dark) │
│   - Multi-Profile Switcher, AI Chat Overlay, Lock Screen    │
└──────────────────────────────┬──────────────────────────────┘
                               │ Tauri v2 IPC (Events/Commands)
┌──────────────────────────────▼──────────────────────────────┐
│                    Tauri Native App Shell                   │
│   - `caf-app`: Fine-grained ACL capabilities and permissions│
│   - Cross-platform window constraints and mobile viewport   │
└──────────────────────────────┬──────────────────────────────┘
                               │ Rust Internal Crate API
┌──────────────────────────────▼──────────────────────────────┐
│                    Rust Crates Ecosystem                    │
│  ┌────────────────────────┐    ┌─────────────────────────┐  │
│  │       `caf-core`       │    │       `caf-xtask`       │  │
│  │ - SQLCipher Storage    │    │ - Code generation engine│  │
│  │ - Master AI Router     │    │ - `new-app` scaffolding │  │
│  │ - Scrubber & PII Guard │    │ - Manifest validation   │  │
│  │ - GCC Billing Client   │    └─────────────────────────┘  │
│  │ - Multi-Profile RBAC   │    ┌─────────────────────────┐  │
│  │ - Sync-ready Schema    │    │       `caf-cli`         │  │
│  │ - Change-log Triggers  │    │ - CLI harness utility   │  │
│  └────────────────────────┘    └─────────────────────────┘  │
└──────────────────────────────┬──────────────────────────────┘
                               │ Encrypted File I/O
┌──────────────────────────────▼──────────────────────────────┐
│       Encrypted Storage Layer (SQLCipher + Argon2id)        │
│   - Local-first database with UUIDv7, integer revision tags │
│   - Auto-updating `change_log` tables for generic sync      │
└─────────────────────────────────────────────────────────────┘
```

## 2. Workspace Crates
- `crates/caf-core`: Core enterprise domain logic.
  - `db.rs` & `vault.rs`: SQLCipher SQLite engine with Argon2id key derivation.
  - `ai/`: Master AI router with multi-provider dispatcher, PII scrubber, context builder, and FTS5 habit memory.
  - `billing/`: Generic GCC Billing Hub client (Mayar IDR & Ko-fi/Stripe USD integrations).
  - `profiles.rs` & `rbac.rs`: Multi-profile partitioning, ownership tags, and visibility flags.
  - `sync.rs`: Generic conflict resolution attributes (UUIDv7, integer revision counter).
- `crates/caf-app`: Tauri v2 application shell and IPC commands registration.
- `crates/caf-cli`: Command line tool (`caframeworkctl`).
- `crates/caf-xtask`: Developer automation tool for codegen and app scaffolding.
- `crates/caf-migrator`: Schema migration and evolution utility.

## 3. Developer Workflow & Extensibility
- App configuration is driven by `app.toml`.
- Running `cargo run -p caf-xtask -- codegen` compiles navigation routes, icons, and capabilities automatically.
- Running `cargo run -p caf-xtask -- new-app --config <config.toml> --out <dest>` generates clean standalone production applications inheriting the full security and AI foundation.
