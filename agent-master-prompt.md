# MASTER PROMPT — CAFramework v2 Feature Completion (1 Orkestrator + 10 Sub-Agent)

> Tempel seluruh isi file ini sebagai prompt awal ke AI agent orchestrator kamu (mis. Claude Code multi-agent / Devin / Cline Team).
> Prompt ini SENGAJA ditulis berdasarkan kondisi nyata repo `D:\Project\caframework` per 2026-09-21 (bukan asumsi generik), supaya agent tidak menebak-nebak atau membangun ulang yang sudah ada.

---

## 0. IDENTITAS PROJECT (JANGAN DIUBAH TANPA IZIN OWNER)

- **Nama**: CAFramework v2 — SSH/terminal client desktop, kelas Termius/Termique.
- **Stack nyata** (verifikasi sendiri sebelum kerja, jangan percaya README lama — README masih template Wails yang belum diupdate):
  - Backend: Rust workspace — `crates/caf-core` (logic inti: vault, ssh, sftp, tunnels, monitor, keys, groups, snippets, backup), `crates/caf-cli` (`caframeworkctl`, saat ini cuma `main.rs` + `bench.rs`), `crates/caf-app` (Tauri v2 shell, `commands.rs` = jembatan Tauri command → core).
  - Frontend: SvelteKit 5 + TailwindCSS 4 + xterm.js, di `frontend/`. Routing file-based di `frontend/src/routes/*`.
  - Packaging: **Tauri**, bukan Wails/Electron. Output akhir wajib installer/`.exe` Windows dari `cargo tauri build` (bundle NSIS di `crates/caf-app/target/release/bundle/nsis/*.exe` atau `.msi`).
- **Dokumen hidup yang WAJIB dibaca dan WAJIB di-update oleh semua agent**: [`ledger-v2.md`](ledger-v2.md) (audit realita, status SELESAI/REGRESI, keputusan desain) dan [`task-v2.md`](task-v2.md) (checklist fitur). Jangan buat dokumen tracking baru — lanjutkan konvensi yang sudah ada, tulis dalam Bahasa Indonesia seperti entri sebelumnya.
- **Referensi visual target**: screenshot aplikasi "Termique" yang diberikan owner. Sidebar acuan, urutan wajib sama:
  `Hosts → Groups → Snippets → Teams → Port forwarding → Monitoring → Command logs → Investigations → SSH keys`
  Plus top bar: notifikasi (bell), theme toggle (sun/moon), status sinkronisasi ("● just now"), window controls custom (bukan native chrome default), dan profile card di pojok kiri bawah (avatar, nama, badge plan "FREE", email).

## 1. MASALAH YANG HARUS DISELESAIKAN

Owner sudah mulai membangun aplikasi ini tapi **belum selesai** dan sejauh ini banyak yang baru UI shell. Tugas kalian: **selesaikan semua fitur sampai benar-benar berfungsi end-to-end dan bisa dipakai**, bukan sekadar tampilan. Bukti "selesai" = ada backend Rust nyata, data tersimpan permanen (SQLite/vault terenkripsi), teruji otomatis, dan bisa dipakai dari UI tanpa data palsu/hardcoded.

### Audit gap saat ini (per commit `50c6f04`, verifikasi ulang sebelum mulai — kondisi bisa berubah):
| Item sidebar referensi | Status di repo | Route frontend | Catatan |
|---|---|---|---|
| Hosts | Ada, kemungkinan besar sudah jalan | `frontend/src/routes/+page.svelte` | Verifikasi: multi-select, "connect", "details" toolbar seperti screenshot |
| Groups | Ada | `routes/groups` | Verifikasi wiring real ke `caf-core/src/groups.rs` |
| Snippets | Ada | `routes/snippets` | Verifikasi eksekusi ke sesi terminal aktif |
| **Teams** | **TIDAK ADA route, tidak ada backend** | — | Fitur baru, harus dibangun full-stack |
| Port forwarding | Ada | `routes/port-forwarding` | Backend `tunnels.rs` — verifikasi local/remote/dynamic semua jalan |
| Monitoring | Ada | `routes/monitoring` | Backend `monitor.rs` — polling agentless via SSH exec |
| **Command logs** | **TIDAK ADA route, tidak ada backend audit log** | — | Fitur baru |
| **Investigations** | **TIDAK ADA route/backend** | — | Fitur baru |
| SSH keys | Ada | `routes/ssh-keys` | Backend `keys.rs`, deploy public key — verifikasi |
| `caframeworkctl` CLI | **REGRESI** — belum ada subcommand host/vault/connect/tunnel/monitor/key/audit (lihat `ledger-v2.md` A-4) | — | Harus dilengkapi agar CLI = fitur GUI |
| `ssh.rs` pakai `ssh2` sync, bukan `russh` async | **REGRESI** (`ledger-v2.md` A-2) | — | Keputusan desain: migrasi bisa ditunda tapi HARUS diputuskan ulang oleh Agent 1, bukan diabaikan diam-diam |

## 2. ATURAN KERAS (NON-NEGOTIABLE)

1. **Dilarang UI palsu.** Setiap tombol/form yang terlihat di layar wajib terhubung ke Tauri command Rust nyata yang membaca/menulis SQLite/vault. Data mock/hardcoded di frontend = gagal, harus diulang.
2. **Local-first & zero-knowledge tetap dipegang.** Jangan tambahkan fitur yang mengirim kredensial/host ke server eksternal manapun. "Cloud Sync E2EE" tetap placeholder disabled sesuai `task-v2.md` §8 — jangan diimplementasikan diam-diam sebagai fitur nyata tanpa izin owner eksplisit.
3. **Definition of Done per fitur** (checklist wajib sebelum ditandai `[SELESAI]` di `task-v2.md`):
   - [ ] Backend Rust: struct + command Tauri + persist ke SQLite (lihat pola di `caf-core/src/db.rs`, `store.rs`).
   - [ ] Terhubung ke UI SvelteKit, reachable dari sidebar nav sesuai urutan referensi.
   - [ ] Minimal 1 unit test Rust (`cargo test -p caf-core`) dan, kalau relevan, 1 skenario manual/E2E didokumentasikan.
   - [ ] `cargo clippy --workspace` 0 warning, 0 error.
   - [ ] `npm run check` (svelte-check) di `frontend/` bersih.
   - [ ] Tidak ada `.unwrap()` baru yang bisa panic dari input user (ikuti pola `run_blocking` helper yang sudah ada di `commands.rs`).
   - [ ] Entri baru ditambahkan ke `ledger-v2.md` (format tabel yang sudah ada) dan checkbox di `task-v2.md` di-centang.
4. **Git hygiene**: setiap sub-agent kerja di branch sendiri (`feat/<area>`, mis. `feat/teams`, `feat/command-logs`), commit kecil & jelas, JANGAN force-push, JANGAN merge ke `2.x` tanpa lolos build+test. Orchestrator (Agent 0) yang melakukan integrasi akhir.
5. **Tidak menyentuh `Cargo.toml` workspace / versi `2.0.9`** kecuali menambah dependency baru yang benar-benar dibutuhkan — dan itu harus dicatat alasannya di `ledger-v2.md`.
6. **Output akhir wajib bisa di-build jadi installer Windows** (`.exe`/`.msi`) yang terinstall dan jalan — ini bukan opsional, ini kriteria lulus dari seluruh proyek.
7. Jika ada ambiguitas desain (misalnya bentuk konkret fitur "Investigations"), sub-agent terkait membuat keputusan desain wajar, mencatatnya sebagai entri baru di bagian "Keputusan Desain (Membutuhkan Konfirmasi Pemilik)" di `ledger-v2.md`, lalu **lanjut jalan** (jangan berhenti menunggu) — konsisten dengan pola kerja yang sudah dipakai di ledger (contoh DX-1, DX-2).

## 3. STRUKTUR TIM: ORCHESTRATOR + 10 SUB-AGENT

### Agent 0 — Orchestrator / Lead Architect (kamu, agent utama)
- **Tidak menulis fitur.** Tugas: memecah kerja ke 10 sub-agent di bawah, jalankan yang independen secara paralel, jalankan yang dependen secara berurutan, resolve conflict antar branch, jaga `ledger-v2.md`/`task-v2.md` tetap konsisten sebagai satu sumber kebenaran, dan **Agent 10 (QA/Release) dijalankan terakhir setelah semua sub-agent lain melapor selesai.**
- Setelah semua fitur (Bagian 4) `[SELESAI]`, orchestrator memicu build final dan melaporkan path `.exe` ke owner.

### Agent 1 — SSH Core & Transport (`crates/caf-core/src/ssh.rs`)
- Audit ulang status `ssh2` vs `russh` (ledger A-2). Putuskan: migrasi penuh ke `russh` (async native) ATAU dokumentasikan secara eksplisit kenapa tetap `ssh2` untuk rilis ini — jangan biarkan status "REGRESI" tanpa keputusan.
- Pastikan TOFU host-key verification (`known_hosts`) tetap utuh setelah perubahan apapun.
- Sediakan interface stabil yang dipakai Agent 4/5/7 (tunnels, monitor, command-logs) tanpa breaking change.

### Agent 2 — CLI Parity (`crates/caf-cli`)
- Implementasikan subcommand `caframeworkctl`: `host` (list/add/rm/connect), `vault` (unlock/lock/status), `connect <host>`, `tunnel` (start/stop/list), `monitor <host>`, `key` (list/add/deploy), `audit` (tail command logs).
- Semua subcommand memanggil `caf-core` langsung (tanpa duplikasi logic), agar GUI dan CLI selalu konsisten.
- Uji manual: `cargo run -p caf-cli -- host list` dsb harus benar-benar berfungsi terhadap vault lokal yang sama dengan GUI.

### Agent 3 — Fitur Baru: **Teams**
- Backend baru `caf-core/src/teams.rs`: model Team (nama, warna/avatar, daftar member lokal, daftar host/group yang di-assign — semua **local-first**, bukan cloud).
- Tauri commands di `commands.rs`, API client `frontend/src/lib/api/teams.ts`.
- Route baru `frontend/src/routes/teams/+page.svelte`, gaya card sama seperti Hosts/Groups di screenshot referensi (grid card, avatar, jumlah anggota).
- CRUD penuh: create/edit/delete team, assign/unassign host ke team.

### Agent 4 — Fitur Baru: **Command Logs**
- Backend baru `caf-core/src/audit.rs` (atau perluas modul yang relevan): rekam setiap event penting ke tabel SQLite terenkripsi — command yang dieksekusi di terminal (via hook di sesi SSH/PTY), eksekusi snippet, buka/tutup tunnel, unlock/lock vault, deploy SSH key.
- Route baru `frontend/src/routes/command-logs/+page.svelte`: list scrollable, filter by host/tanggal/jenis event, search text, export ke file (lihat pola `backup.ts`/`backup.rs` untuk cara export terenkripsi kalau perlu).
- Perhatikan privasi: jangan log isi password/secret — mask otomatis.

### Agent 5 — Fitur Baru: **Investigations**
- Backend baru `caf-core/src/investigations.rs`: konsep "investigation" = kumpulan bukti tersimpan (saved search/filter dari Command Logs + snapshot Monitoring + catatan manual/timeline) yang bisa dinamai dan disimpan untuk audit/insiden.
- Relasi ke data Agent 4 (Command Logs) — koordinasi lewat Agent 0 supaya skema tabel tidak bentrok.
- Route baru `frontend/src/routes/investigations/+page.svelte`: list investigation tersimpan, detail view dengan timeline events, tombol "New Investigation" (pilih rentang waktu + host terkait, auto-tarik command logs relevan).

### Agent 6 — Penuntasan: Hosts, Groups, Snippets, SSH Keys
- Bukan bikin baru — **audit & tuntaskan** 4 fitur yang diklaim `[x]` di `task-v2.md` tapi perlu diverifikasi nyata jalan (bukan hanya render UI).
- Cocokkan toolbar Hosts dengan screenshot: mode `select` (multi-select), `connect` (klik langsung sambung), `details` (buka panel info) — pastikan ketiganya benar-benar berfungsi, bukan dekorasi.
- Pastikan Add/Edit/Delete Host, Group multiselect-host, Snippet 2-step modal, SSH key deploy — semuanya real round-trip ke SQLite/vault dan bertahan setelah restart app.

### Agent 7 — Penuntasan: Port Forwarding & Monitoring
- Verifikasi `tunnels.rs` (local/remote/dynamic) benar-benar membuka socket dan bisa dites (mis. `curl` lewat local forward ke server test).
- Verifikasi `monitor.rs` menampilkan data live (CPU/RAM/Disk/OS/Uptime) yang update berkala, bukan angka statis — cocokkan dengan indikator hijau "● just now" di top bar referensi (status sinkronisasi/live).
- Tangani edge case: host offline, auth gagal, koneksi putus di tengah tunnel aktif — harus ada UI error state yang jelas, bukan silent fail.

### Agent 8 — Security & Vault Hardening
- Verifikasi klaim di `ledger-v2.md` (Argon2id m=64MB t=3 p=4, zeroize, AES-256-GCM backup) dengan test nyata — jangan percaya begitu saja, baca kode `vault.rs`/`secret.rs`/`backup.rs`.
- Tuntaskan item `task-v2.md` §9: pastikan SQLite/field sensitif benar-benar terenkripsi at-rest (bukan cuma field password, cek semua kolom secret), minimum 8 karakter password vault ditegakkan di UI+backend (dua-duanya, jangan cuma frontend).
- Test roundtrip export/import backup (`T2-EXIM-01`) dengan data asli, pastikan tidak ada kebocoran plaintext di file backup.

### Agent 9 — Frontend Shell & UX Parity
- Update sidebar nav (`+layout.svelte` dan komponen terkait) supaya urutan & item PERSIS: Hosts, Groups, Snippets, Teams, Port forwarding, Monitoring, Command logs, Investigations, SSH keys — termasuk icon yang sesuai konteks masing-masing.
- Top bar: bell notification (hubungkan ke `NotificationCenter.svelte` yang sudah ada — pastikan isinya nyata, bukan kosong), theme toggle (pastikan dark/light benar-benar switch semua halaman baru), status "● just now" (live indicator, update timestamp asli).
- Profile card kiri-bawah: avatar, nama dari `authors` config atau profil user lokal, badge "FREE" (siapkan tempat untuk badge plan lain kalau nanti ada tier), email.
- Pastikan window controls custom (minimize/maximize/close) tetap berfungsi di window baru manapun yang ditambahkan (dialog/modal Teams, Command Logs, Investigations).

### Agent 10 — QA, Testing & Release Engineer (jalan TERAKHIR)
- Jalankan penuh: `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `npm run check` di `frontend/`.
- Uji manual tiap fitur baru (Teams, Command Logs, Investigations) end-to-end dengan skenario nyata (host test SSH sungguhan, bukan mock).
- Jalankan `cargo tauri build` (dari `crates/caf-app` atau root sesuai `tauri.conf.json`), verifikasi installer `.exe`/`.msi` ter-generate di `target/release/bundle/`, **install ulang di mesin bersih/VM kalau memungkinkan**, jalankan app hasil install, screenshot bukti semua 9 menu sidebar berfungsi.
- Update final `ledger-v2.md` dengan tanggal audit baru dan tabel status akhir semua item di Bagian 1 tabel gap.
- Laporkan ke Agent 0: path file `.exe` final + ringkasan fitur yang masih punya known-issue (kalau ada) untuk didiskusikan ke owner.

## 4. URUTAN EKSEKUSI YANG DISARANKAN

1. **Paralel gelombang 1** (tidak saling bergantung): Agent 1 (SSH core), Agent 6 (audit fitur lama), Agent 8 (security hardening), Agent 9 (shell/nav — bisa mulai dari mockup nav dulu).
2. **Paralel gelombang 2** (butuh Agent 1 stabil dulu untuk hal yang sentuh ssh session): Agent 3 (Teams — independen, bisa mulai kapan saja), Agent 4 (Command Logs — butuh hook ke sesi PTY dari Agent 1), Agent 7 (Port Forwarding & Monitoring lanjutan).
3. **Gelombang 3**: Agent 5 (Investigations — butuh skema Command Logs dari Agent 4 sudah stabil), Agent 2 (CLI parity — butuh semua core module final agar tidak bolak-balik ubah signature).
4. **Final**: Agent 0 integrasi semua branch ke `2.x`, lalu Agent 10 jalan build & QA menyeluruh.

## 5. FORMAT LAPORAN SETIAP SUB-AGENT KE ORCHESTRATOR

Setiap sub-agent WAJIB lapor dengan format ini sebelum ditandai selesai:
```
AGENT: <nomor & nama>
FITUR: <nama fitur>
FILE DIUBAH/DITAMBAH: <daftar path>
STATUS: SELESAI | BLOCKED | PARTIAL
TEST: <command yang dijalankan + hasil>
LEDGER UPDATE: <baris yang ditambahkan ke ledger-v2.md>
CATATAN/RISIKO: <kalau ada>
```

## 6. KRITERIA LULUS AKHIR (SEBELUM MELAPOR "SELESAI" KE OWNER)

- [ ] Sembilan menu sidebar referensi semuanya ada, berfungsi, dan menyimpan data secara persisten.
- [ ] `caframeworkctl` punya parity command dengan fitur GUI utama.
- [ ] `cargo test --workspace` dan `cargo clippy --workspace` bersih.
- [ ] `npm run check` (frontend) bersih.
- [ ] `cargo tauri build` menghasilkan installer `.exe`/`.msi` yang **berhasil diinstall dan dijalankan**.
- [ ] `ledger-v2.md` dan `task-v2.md` mencerminkan status akhir yang benar (tidak ada klaim `[SELESAI]` palsu).
- [ ] Tidak ada kredensial/plaintext bocor di database, backup file, atau log.
