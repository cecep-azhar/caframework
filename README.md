# CAFramework v2

> **High-Performance, Zero-Knowledge, Multi-Platform SSH Terminal & DevOps Client**  
> Built primarily with **Rust** (Tauri v2 + Tokio + SQLCipher) and SvelteKit.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-brightgreen.svg)](#supported-platforms)
[![Rust](https://img.shields.io/badge/Core-Rust%202024-orange.svg)](https://www.rust-lang.org/)
[![GitHub Issues](https://img.shields.io/badge/Issues-GitHub-blue)](https://github.com/cecep-azhar/caframework/issues)
[![Support on Ko-fi](https://img.shields.io/badge/Support-Ko--fi-FF5E5B?logo=ko-fi&logoColor=white)](https://ko-fi.com/cecepazhar)

---

## ⚡ Overview

**CAFramework** is a next-generation desktop terminal emulator and server management suite engineered in **Rust** for supreme speed, rock-solid stability, and zero-compromise privacy. It combines an ultra-responsive raw PTY SSH client, full WinSCP-parity dual-pane SFTP manager, real-time remote system metrics monitoring, and intelligent AI-assisted server operations into a sleek, dark-mode native desktop application.

Unlike web-wrapped shells or bloated electron tools, CAFramework runs directly on lightweight Rust native binaries with memory footprint strictly optimized under 180MB RSS using `mimalloc` and headless WebKit rendering.

---

## 🖥️ Supported Platforms

CAFramework is built from the ground up for **Multi-Platform** deployment:

| Platform | Packages / Bundles | Status |
| :--- | :--- | :--- |
| **Linux (Fedora / RHEL)** | `.rpm` (x86_64, aarch64) | ✅ Active & Tested |
| **Linux (Debian / Ubuntu)** | `.deb` (x86_64, aarch64) | ✅ Active & Tested |
| **Linux (Universal)** | `.AppImage` | ✅ Active & Tested |
| **Windows 10 / 11** | Native `.exe` Installer & Portable (x64) | ✅ Active & Tested |
| **Android** | `.apk` (ARM64) | 🚧 In Progress |

---

## 🚀 Key Features

### 1. ⚡ Zero-Delay SSH PTY Terminal
- Built on low-level Rust `ssh2` and Tokio non-blocking async streaming.
- Immediate byte-by-byte PTY flush — zero input delay or typing latency even on high-speed typing and heavy terminal output.
- Full xterm.js compatibility with 256 colors, true color, and native copy/paste.

### 2. 🔄 True Multi-Session Persistence
- Seamlessly open, switch, and split SSH sessions across multiple remote hosts without losing your session state.
- Adding a new host connection never disconnects or restarts background sessions; your remote processes, long builds, and terminal scrollbacks remain alive and uninterrupted.

### 3. 📁 Dual-Pane SFTP Manager (WinSCP Parity)
- Integrated side-by-side local filesystem and remote SFTP file browsers.
- Chunked 64KB high-speed stream upload/download with real-time progress bars and cancel tokens.
- Native keyboard shortcuts (`F2` Rename, `F5` Copy, `F7` New Folder, `F8` Delete, `Ctrl+R` Refresh).
- Visual permissions viewer and chmod editor.

### 4. 🧠 Smart Autocomplete & Autosuggest
- Fish/Zed-style ghost autocompletion right on the active terminal cursor.
- Suggests 200+ curated Linux sysadmin utilities, saved personal vault snippets, and active session command history.
- Press `Tab` or `Arrow Right` to immediately apply commands.

### 5. 🔒 Zero-Knowledge Encrypted Vault
- All sensitive credentials (passwords, private SSH keys, AI API tokens) are encrypted locally using **SQLCipher (AES-256-GCM)** and **Argon2id** key derivation.
- Zero telemetry, zero tracking, zero remote key upload. Your keys never leave your device.
- Full encrypted backup and restore (`.catb` format) with passphrase verification.

### 6. 📊 Real-Time Server Monitoring
- Direct non-interactive SSH telemetry polling.
- Live charts and resource monitors for CPU load, RAM utilization, Disk usage, and Network I/O without needing third-party agents on target machines.

### 7. 🌐 Port Forwarding & SSH Tunnels
- Easily configure and manage Local, Remote, and Dynamic (SOCKS5 proxy) tunnels.
- Single-click tunnel activation with live port status indicators.

### 8. 🤖 AI Ops Copilot (Local & OpenAI-Compatible)
- Execute natural-language infrastructure tasks safely.
- Transparent step-by-step proposal review before running any bash commands.
- Full E2EE audit logging of every terminal command and AI execution.

---

## 🛠️ Architecture

CAFramework is structured as a clean Rust workspace:

```text
caframework/
├── crates/
│   ├── caf-core/      # Pure Rust core: Vault (SQLCipher), SSH engine, SFTP, monitoring, store
│   ├── caf-cli/       # CLI utility for terminal testing, benchmarks, and vault migrations
│   └── caf-app/       # Tauri v2 native desktop application wrapper (IPC, menus, system tray)
├── frontend/             # SvelteKit + Tailwind CSS single-page application (bundled into Tauri)
└── scripts/              # Automated build, packaging, and memory budget validation tools
```

---

## 📦 Building from Source

### Prerequisites
- **Rust toolchain** (1.80+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js** (v18+ or v20+) & **npm**
- **System dependencies**:
  - *Fedora / RHEL*: `sudo dnf install webkit2gtk4.1-devel openssl-devel libssh2-devel libappindicator-gtk3-devel`
  - *Ubuntu / Debian*: `sudo apt install libwebkit2gtk-4.1-dev libssl-dev libssh2-1-dev libayatana-appindicator3-dev`
  - *Windows*: Visual Studio C++ Build Tools & WebView2 runtime

### 1. Clone the Repository
```bash
git clone https://github.com/cecep-azhar/caframework.git
cd caframework
```

### 2. Install Frontend Dependencies
```bash
cd frontend
npm install
npm run build
cd ..
```

### 3. Run in Development Mode
```bash
cd crates/caf-app
npx --prefix ../../frontend tauri dev
```

### 4. Build Production Packages

#### Linux (.rpm / .deb / .AppImage)
```bash
cd crates/caf-app
# Build RPM bundle
npx --prefix ../../frontend tauri build --bundles rpm

# Build DEB bundle
npx --prefix ../../frontend tauri build --bundles deb

# Build AppImage bundle
npx --prefix ../../frontend tauri build --bundles appimage
```

#### Windows (.exe)
```bash
cd crates/caf-app
npx --prefix ../../frontend tauri build --bundles nsis
```

---

## 🤝 Community & Support

- **Repository**: [https://github.com/cecep-azhar/caframework](https://github.com/cecep-azhar/caframework)
- **Issue Tracker & Feature Requests**: [https://github.com/cecep-azhar/caframework/issues](https://github.com/cecep-azhar/caframework/issues)
- **Support & Sponsorship**: [ko-fi.com/cecepazhar](https://ko-fi.com/cecepazhar) (primary) · [GitHub Sponsors](https://github.com/sponsors/cecep-azhar) · [PayPal](https://paypal.me/cecepazhar)

---

## 💖 Sponsors

CAFramework's core stays free and MIT-licensed forever. Sponsorship pays for servers, cross-platform builds, and the time to keep shipping updates.

| Tier | Monthly | You get |
| :--- | :--- | :--- |
| 💎 **Platinum** | $100+ | Large logo at the top of this section, on [caframework.fathforce.com](https://caframework.fathforce.com) and in the docs · one CAFramework Pro license · quarterly roadmap call |
| 🥇 **Gold** | $25+ | Logo in this section and on the website · thank-you in release notes · priority issue triage |
| 🥈 **Silver** | $5+ | Your name in the supporters list below and on the website |

[![Become a sponsor on Ko-fi](https://img.shields.io/badge/Become%20a%20sponsor-Ko--fi-FF5E5B?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/cecepazhar)

### 💎 Platinum

_Your logo here — [become the first Platinum sponsor](https://ko-fi.com/cecepazhar)._

### 🥇 Gold

_Your logo here — [become a Gold sponsor](https://ko-fi.com/cecepazhar)._

### 🥈 Silver

_Your name here — [become a Silver supporter](https://ko-fi.com/cecepazhar)._

---

## 📄 License

Licensed under the [MIT License](LICENSE). Free and open source; CAFramework Pro only covers server-backed features (Cloud Sync, hosted AI).
