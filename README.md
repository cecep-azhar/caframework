# CAFramework

Modern, sync-ready, multi-profile application starter framework for Desktop (Linux, macOS, Windows) and Mobile (Android).

Built with **Rust (Edition 2024)**, **Tauri v2**, **SQLCipher (Argon2id)**, **Svelte 5 Runes**, and **Tailwind CSS v4**.

---

## ✨ Features
- 🔒 **SQLCipher Encrypted Local Storage** with Argon2id Master Key & PIN hashing.
- 👥 **Multi-Profile Architecture** with roles, ownership, and visibility access control.
- 🔄 **Sync-Ready Generic Schema** (UUIDv7, integer `rev`, automatic `change_log` audit triggers).
- ⚡ **Svelte 5 Runes Frontend** with instant reactivity, dark mode, dynamic accent colors.
- 🤖 **Generic AI Assistant Slice** with privacy regex scrubber (`[REDACTED_EMAIL]`, `[REDACTED_ACCOUNT]`).
- 🛠️ **`cargo xtask` Tooling**: Single manifest `app.toml` drives identity, navigation codegen, and `new-app` scaffolding.
- 📱 **Android Ready** with complete mobile documentation and signing guides.

---

## 🚀 Quick Start

### 1. Requirements
- Rust 1.85+ (Edition 2024)
- Node.js 20+ & npm

### 2. Setup & Development
```bash
# Clone repository
git clone https://github.com/cecepazhar/caframework.git
cd caframework

# Run Codegen
cargo run --package caf-xtask -- codegen

# Build & run in dev mode
cargo tauri dev
```

### 3. Generate New Derivative App
```bash
cargo run --package caf-xtask -- new-app --config my-app.toml --out ~/Project/my-app
```

---

## 📖 Documentation
- [Architecture Overview](docs/architecture.md)
- [Creating a New App](docs/creating-an-app.md)
- [Adding a New Menu / Slice](docs/adding-a-menu.md)
- [Android Build & Signing Guide](docs/android-build-guide.md)
- [Error Codes Taxonomy](docs/error-codes.md)

---

## 📄 License & Attribution
CAFramework is open-source under the [MIT License](LICENSE).
See [NOTICE](NOTICE) for original project attribution.
