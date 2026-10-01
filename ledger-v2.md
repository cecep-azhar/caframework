# CAFramework v2 Ledger & Audit

## Audit Realita (19 Sep 2026)
| # | Pertanyaan | Status Aktual |
|---|---|---|
| A-1 | Vault Argon2id sungguhan? | `[SELESAI]` Modul KEK menggunakan Argon2id dan Zeroize diterapkan. |
| A-2 | `ssh.rs` memakai `russh` atau `ssh2`? | `[SELESAI]` Tetap memakai `ssh2` untuk rilis 2.x demi stabilitas modul lain (tunnels/monitor/SFTP) yang bergantung padanya. Interface asinkron disimulasikan lewat tokio mpsc & thread worker. |
| A-3 | TOFU host-key verification? | `[SELESAI]` Diimplementasikan di `caf-core/src/ssh.rs` dengan pencatatan & verifikasi `known_hosts` otomatis! |
| A-4 | `caframeworkctl` CLI commands? | `[SELESAI]` Tersedia subcommands untuk host, vault, connect, tunnel, monitor, key, dan audit. Semua memanggil langsung fungsi `caf-core`. |
| A-5 | SFTP (Fase 4)? | `[SELESAI]` Ditambahkan via `crates/caf-core/src/sftp.rs` beserta antarmuka UI. |

## Utang Teknis & Task Selesai (19 Sep 2026)
- [x] **Clippy Cleanup**: 14 titik `.unwrap()` di `crates/caf-app/src/commands.rs` diatasi dengan `run_blocking` helper. `cargo clippy --workspace` kini lulus 0 error/0 warning!
- [x] **`T2-SSH-04` — TOFU Host Key Verification**: Menambahkan verifikasi `known_hosts` otomatis pada `ssh::connect`. Celah MITM berhasil ditutup.
- [x] **`T2-TOOL-05` — SSH Keys Manager**: Implementasi backend Rust (`caf-core/src/keys.rs`) & UI SvelteKit (`/ssh-keys`). Private key disimpan terenkripsi AES-256-GCM di SQLite database lokal.
- [x] **`T2-TOOL-06` — Deploy public key**: Fitur `Deploy to Server` di UI SSH Keys. Deploy idempoten ke `~/.ssh/authorized_keys` menggunakan backend `ssh2` channel exec.
- [x] **`T2-TOOL-02` — Port Forwarding**: Manajer tunneling dengan mode *Local*, *Remote*, dan *Dynamic*. Hardened socket bindings dan synchronous error reporting. Terikat ke ID host. Otomatis membersihkan socket *direct-tcpip* saat ditutup. Backend Rust `caf-core/src/tunnels.rs` + UI `/port-forwarding`.
- [x] **`T2-TOOL-07` — Server Monitoring**: Menampilkan pemakaian real-time (CPU, RAM, Root Disk, OS, Uptime) pada tab Monitoring untuk seluruh host aktif. Menggunakan skrip *agentless* Unix yang di-_poll_ via SSH `exec`. Hardened with SSH session timeouts. Terintegrasi dengan sinkronisasi "just now" global di top bar UI. Backend Rust `caf-core/src/monitor.rs` + UI SvelteKit `/monitoring`.
- [x] **`T2-SFTP-01` — SFTP / Remote File Manager**: Implementasi di `sftp.rs` untuk baca/tulis/hapus file jarak jauh secara efisien lewat PTY koneksi SSH yang sedang aktif. UI disediakan di tab SvelteKit `/sftp`.
- [x] **`T2-CORE-01` — Argon2id Vault (Phase 1)**: Modul KEK `vault.rs` menggunakan algoritma sandi `argon2` m=64MB, t=3, p=4. Kunci memori dihapus otomatis dengan `zeroize` saat app di-lock. Celah fallback `vault.key` plaintext dihapus.
- [x] **`T2-EXIM-01` — Ekspor/Impor Kredensial**: Encrypted JSON Vault Backup (`backup.rs`). Kunci turunan diturunkan (Argon2id) dari `Backup Passphrase`. Import mendeskripsi format secara aman. Telah diuji dengan real data untuk memastikan tidak ada plaintext leakage (T2-EXIM-01 roundtrip test). Didukung UI Settings > *Backup & Restore*. Minimum password 8 karakter diterapkan pada UI & Backend.

## Keputusan Desain (Membutuhkan Konfirmasi Pemilik)
* **DX-1 (ssh2 vs russh)**: Default -> **DIPUTUSKAN**. Kita tetap menggunakan `ssh2` pada rilis 2.x. Thread worker dan mpsc channels sudah memberikan antarmuka asinkron yang cukup stabil untuk modul lain (Agent 4/5/7: tunnels, monitor, sftp, logs) tanpa menyebabkan breaking changes pada rilis ini. TOFU berjalan baik via libssh2 native `known_hosts`.
* **DX-2 (Kerjakan TOFU sebelum Power Tools?)**: Default -> **SELESAI DULUAN**. Verifikasi host key TOFU (`REQ-18`/`T2-SSH-04`) telah selesai dikerjakan sebelum SSH Keys Manager.- [x] **Audit & Fix (21 Sep 2026)**: Verified that Hosts, Groups, Snippets, and SSH Keys fully persist to SQLite vault via caf-core and work across restarts. Updated Hosts UI toolbar to include select mode, connect button, and details panel button.
- [x] **Teams Feature (21 Sep 2026)**: Added Teams model to SQLite (T6/Teams) mapping to multiple local members, hostIds, and groupIds. Included Tauri commands and SvelteKit route with the Hosts/Groups UI style.

- [x] **`T2-CORE-02` — Command Logs (Audit)**: Implemented full-stack audit logs for PTY terminal commands, tunnel start/stop, vault lock/unlock, and SSH key deploy. Logs are saved in `caframework.db` and masked for secrets using regex. SvelteKit UI in `/command-logs` with filtering, search, and CSV export.
- [x] **Investigations Feature (21 Sep 2026)**: Added full-stack Investigations feature (Model in `caf-core/src/investigations.rs` with `id, title, host_id, status, notes, evidence`). Persisted to `caframework.db`. Built UI timeline in `frontend/src/routes/investigations/+page.svelte` aligned with Agent 4's Audit log design.

- [x] **Prompt Studio & AI Ops Assistant (21 Sep 2026)**:
  - **AI Planning Engine**: Modul Rust di `caf-core/src/ai.rs` dengan schema `ai_settings`, `ai_execution_plans`, `ai_plan_steps` di SQLite. Mendukung custom provider (OpenAI, Anthropic, Ollama, 9router) dan built-in fallback heuristic engine dengan preset cerdas (Laravel 11 Dev, Docker, Node.js, UFW Security).
  - **Human-in-the-Loop Confirmation**: Antarmuka konfirmasi langkah eksekusi di `frontend/src/routes/prompt-studio/+page.svelte` dengan seleksi checkbox per langkah, indikator `SUDO / SYSTEM`, dan terminal viewer live.
  - **Automated SSH Execution & Audit Trail**: Eksekusi SSH non-interactive channel per step dengan pelaporan status realtime dan pencatatan otomatis ke `command_logs` (event `AI_AUTOMATION`).

## Final Release Status (21 Sep 2026 - v2.0.10)
- [x] **Quality Gates**: `npm run check` and `npm run build` in `frontend/` passed cleanly (0 errors). Rust workspace tests in `caf-core` (43 unittests, 8 AI tests) passed.
- [x] **Installer Build**: Successfully generated Windows installation packages via `cargo tauri build` for version **2.0.10**:
  - **NSIS Installer (.exe)**: `D:\Project\caframework\target
elease\bundle\nsis\CAFramework_2.0.10_x64-setup.exe`
  - **MSI Installer (.msi)**: `D:\Project\caframework\target
elease\bundle\msi\CAFramework_2.0.10_x64_en-US.msi`
- [x] **Release Status**: `SELESAI` (v2.0.10 final release ready for deployment).
- [x] **UI/UX & Responsiveness Overhaul (21 Sep 2026)**:
  - **Auto-collapsing Icon Sidebar**: Menu sebelah kiri otomatis menciut menjadi *icon-only* saat ada koneksi aktif (`sessionTabs.length > 0`), dilengkapi tombol manual toggle dan tooltip informatif.
  - **Optimasi Luas Layar**: Menghapus padding `p-6` yang boros pada sesi aktif (`p-0 md:p-1`), compact header bar, dan terminal pane memenuhi 100% viewport.
  - **Split Screen In-Connection**: Kontrol tata letak split (Single, Horizontal, Vertical, Grid 2x2) hanya muncul di dalam sesi koneksi multi-host (`tabs.length > 1`).
  - **Penyimpanan Workspace**: Fitur simpan dan aktifkan kembali susunan tab, layout, serta status file explorer remote via `WorkspaceMenu` & `workspaceStore.svelte.ts` (`localStorage['caframework_workspaces_v2']`).
  - **Navigasi Responsif Mobile**: Navigasi slide-out drawer dengan hamburger toggle pada layar kecil (<768px), layout terminal adaptif dan tab switcher khusus mobile agar terminal tetap terbaca jelas.

## WinSCP Feature Parity Architecture Decisions (22 Sep 2026)
* **DX-3 (Protokol Non-SSH & Trait Abstraction)**: Diperkenalkan trait `RemoteFileSystem` di `caf-core` untuk membungkus operasi FS (list, read, write, stat, remove, rename, mkdir). Crate `suppaftp` dipilih untuk FTP/FTPS, `reqwest` untuk WebDAV, dan `rust-s3` (lebih ringan dari `aws-sdk-s3` untuk desktop bundle) untuk S3-compatible storage.
* **DX-4 (Transfer Queue & Resume State)**: Antrean transfer dikelola via background worker `caf-core/src/transfers.rs` dengan persistensi status task di SQLite untuk mendukung resume byte-offset pasca restart.
* **DX-5 (Editor Kode Ringan)**: CodeMirror 6 dipilih menggantikan Monaco untuk memangkas bundle size webview desktop.
* **DX-6 (Integritas, Pencarian, Kompresi)**: Remote operations mengutamakan execution channel cepat berbasis Unix commands (`sha256sum`, `find`, `tar`, `zip`) dengan safe shell escaping, serta fallback client-side streaming jika remote OS minimalis.

