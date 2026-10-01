# CAFramework Architecture

## 1. Overview
CAFramework adalah open-source starter framework untuk aplikasi desktop (Linux, macOS, Windows) dan mobile (Android) yang aman, modular, dan modern berbasis:
- **Rust Backend**: Edition 2024, modular workspace crates (`caf-core`, `caf-app`, `caf-cli`, `caf-xtask`).
- **Storage Engine**: SQLCipher (database terenkripsi AES-256-GCM) dengan Argon2id key derivation.
- **Frontend**: Svelte 5 (Runes reactivity `$state`, `$derived`, `$props`), SvelteKit, Tailwind CSS v4.
- **IPC Architecture**: Tauri v2 dengan ACL fine-grained permission.

---

## 2. Workspace Crates
- `crates/caf-core`: Business logic murni, isolated database, vault encryption, generic slices, crash reporting & scrubber.
- `crates/caf-app`: Tauri v2 application shell, commands registry, window lifecycle.
- `crates/caf-cli`: CLI utility harness (`caframeworkctl`).
- `crates/caf-xtask`: Developer codegen tool & app generator (`cargo xtask codegen`, `cargo xtask new-app`).

---

## 3. Data Flow & Sync Ready Model
Setiap record data generic dilengkapi atribut:
- `id` (UUIDv7 string)
- `rev` (Integer revision counter)
- `created_at` / `updated_at` / `deleted_at` (ISO 8601 UTC)
- `origin_device_id` (Device fingerprint)
- `owner_profile_id` (Multi-profile identifier)
- `visibility` (`private` / `shared`)
- Triggers otomatis mencatat setiap mutasi ke tabel `change_log`.
