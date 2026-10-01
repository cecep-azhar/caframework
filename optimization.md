# CAFramework — Optimization Checklist

Hasil audit cepat repo per 2026-09-24. Sudah cukup banyak yang dituning dengan baik
(lihat "Sudah OK" di tiap seksi) — daftar ini fokus ke gap yang masih ada.

## 1. Binary size & startup

| Item | File | Prioritas | Kenapa |
|---|---|---|---|
| `tokio` pakai feature `"full"` | [crates/caf-app/Cargo.toml:24](crates/caf-app/Cargo.toml:24) | Sedang | `full` menarik semua subsystem (net listener/accept, signal, fs, dll) yang gak semuanya dipakai — kode async murni pakai `spawn_blocking` + thread manual untuk PTY ([ssh.rs](crates/caf-core/src/ssh.rs)). Trim ke `["rt-multi-thread","macros","sync","time","process","io-util"]` (sesuaikan setelah cek compile error) buat mangkas binary size — relevan langsung ke `BUDGET_BINARY_MB=8` di [budget-check.sh](scripts/budget-check.sh). |
| Cek pakai `cargo bloat` / `cargo tree` buat cari dependency duplikat | workspace | Rendah | `ssh2` di-vendor OpenSSL di non-Windows ([Cargo.toml:38](crates/caf-app/Cargo.toml:38)) — kombinasi dengan `webkit2gtk` + `tokio full` berpotensi bawa symbol duplikat. Jalankan `cargo bloat --release -n 20` sekali buat baseline. |

**Sudah OK:** `opt-level="z"`, `lto="fat"`, `codegen-units=1`, `panic="abort"`, `strip=true` ([Cargo.toml:24-29](Cargo.toml)), `mimalloc` sebagai global allocator ([main.rs:14](crates/caf-app/src/main.rs:14)). Gak banyak lagi yang bisa dikorek di level ini.

## 2. WebView / memori runtime

**Sudah OK — jangan diubah tanpa alasan kuat:** `additionalBrowserArgs` di [tauri.conf.json](crates/caf-app/tauri.conf.json) udah rapi: `--renderer-process-limit=1`, `--in-process-gpu`, `disk-cache-size=16MB`, disable Translate/OptimizationHints/MediaRouter/GPU shader disk cache. Ini yang bikin private memory turun dari budget lama 380MB ke ambang 160MB (lihat catatan di [budget-check.sh](scripts/budget-check.sh)).

| Item | Prioritas | Kenapa |
|---|---|---|
| Code-split route berat (`prompt-studio`, `sftp`, `investigations`, `command-logs`) | Sedang | SvelteKit udah otomatis route-split per halaman, tapi cek `npm run build` output (`frontend/build/_app/immutable/chunks`) — pastikan CodeMirror & AI module gak ke-bundle di chunk awal yang dimuat saat cold start layar Dashboard. |
| Lazy-load `@codemirror/lang-*` per ekstensi file | Sedang | 8 bahasa (`css/html/javascript/json/markdown/python/rust`) semuanya static-import di [package.json](frontend/package.json) — kalau di-import langsung di komponen editor (bukan dynamic `import()` sesuai ekstensi file yang dibuka), semua ikut ke initial bundle padahal user biasanya cuma buka 1-2 jenis file per sesi SFTP. |

## 3. Terminal rendering (xterm.js)

| Item | File | Prioritas | Kenapa |
|---|---|---|---|
| Belum pakai renderer WebGL/Canvas | [TerminalPane.svelte:227](frontend/src/lib/components/TerminalPane.svelte:227) | **Tinggi** | `new Terminal({ scrollback: 1000 })` tanpa addon renderer → xterm fallback ke DOM renderer, yang paling boros CPU/memori pas output deras (build log, `cat` file besar, `htop`). Tambah `@xterm/addon-webgl` (dengan fallback `@xterm/addon-canvas` kalau WebGL gagal init di WebView) — biasanya turun signifikan pada beban tinggi, terasa banget di split grid 2x2 (4 instance render bareng). |
| Scrollback per pane di mode split | sama | Rendah | 1000 baris × sampai 4 pane (grid 2x2) = buffer × 4. Kalau device target juga IoT/low-resource, pertimbangkan turunin scrollback default saat `tabs.length > 1` atau lazy-alloc buffer pane yang lagi gak fokus. |

**Sudah OK:** `terminal.dispose()` dipanggil di cleanup `onMount` ([TerminalPane.svelte:416](frontend/src/lib/components/TerminalPane.svelte:416)) dengan guard `disposed` — gak ada leak listener yang jelas dari sisi ini.

## 4. SQLite / Vault I/O

| Item | File | Prioritas | Kenapa |
|---|---|---|---|
| Belum ada `PRAGMA journal_mode=WAL` | [db.rs](crates/caf-core/src/db.rs) | Sedang | Cuma ketemu `PRAGMA key` (SQLCipher) — default journal mode `DELETE` fsync seluruh file tiap transaksi dan blok reader saat writer aktif. Command Logs nulis tiap command PTY dieksekusi ([ledger-v2.md](ledger-v2.md) — Agent 4 audit log), jadi tiap keystroke-command bisa nunggu I/O. Tambah `PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;` setelah `PRAGMA key` — SQLCipher tetap enkripsi file `-wal`-nya, aman, cuma nambah 1 file di direktori data. |

## 5. SSH / Monitoring

**Sudah OK:** `monitor.rs` sengaja pakai pooled non-interactive session (`with_exec_session`), bukan reuse session PTY — ada catatan eksplisit di komentar kenapa (dulu pernah bikin terminal freeze). Gak perlu disentuh.

## 6. Repo & build hygiene (bukan runtime, tapi ngaruh ke clone/CI)

| Item | Prioritas | Kenapa |
|---|---|---|
| `dist/` (409MB) + `.rpm` lama di root repo | Sedang | Installer per-versi numpuk di git (v2.0.0 s/d v2.1.9). Pindahin ke GitHub Releases aja, jangan commit binary hasil build — repo clone jadi berat & `git log` di folder ini lambat. |
| `target/` lokal 35GB | Rendah | Wajar buat workspace 3 crate + `lto=fat` (build release nyimpen banyak artifact intermediate). Bukan masalah shipping, cuma `cargo clean` berkala kalau disk penuh. Pertimbangkan `sccache` buat rebuild lebih cepat tanpa nambah ukuran akhir. |

## Ringkasan prioritas

1. **Tinggi** — xterm WebGL/Canvas addon (dampak paling kerasa ke pengalaman & CPU saat output deras).
2. **Sedang** — trim `tokio` features, WAL mode SQLite, lazy-load CodeMirror lang modes, bersihin `dist/` dari git.
3. **Rendah** — `cargo bloat` audit, scrollback tuning per split mode.

Validasi tiap perubahan pakai harness yang udah ada: `scripts/bench.sh` lalu `scripts/budget-check.sh` buat cek `binary_size_mb`, `cold_start_ms`, `private_mb` gak regresi.
