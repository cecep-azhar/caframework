# Audit Statis CAFramework — 2 Oktober 2026

| | |
| --- | --- |
| Repo | `~/Project/caframework`, branch `main`, HEAD `7ec9e64`, working tree bersih (`git status` kosong), 322 file ter-track |
| Acuan | `prd.md` v1.0, `task.md` v1.0, `prompt-dev.md` (Appendix A) di folder Notes `4 - Product/1 CAFramework` |
| Mode | **Audit statis read-only** sesuai pilihan pemilik. Tidak ada `cargo`/`npm` yang dijalankan dan tidak ada file repo yang diubah. Bukti berupa `file:baris` dan output `grep`/skrip baca-saja. |
| Legenda | **PASS** = terbukti dari kode · **PARTIAL** = sebagian · **FAIL** = tidak ada atau bertentangan dengan PRD · **BELUM DIVERIFIKASI** = perlu dijalankan (lihat §5). Kata *prediksi* = kesimpulan statis yang wajib dikonfirmasi dengan perintah di §5. |

## Ringkasan

**Belum layak dirilis sebagai v0.1.0; Definition of Done PRD §9 belum terpenuhi.** Dari 73 butir yang dinilai: **16 PASS, 16 PARTIAL, 40 FAIL, 1 BELUM DIVERIFIKASI**. Ada 1 temuan Kritis dan 10 temuan Tinggi.

Yang sudah benar: crate SSH/FTP/PTY dan paket `@xterm/*` benar-benar hilang dari dependency, vault memakai SQLCipher + Argon2id, hash PIN Argon2id disimpan di dalam vault, tabel `notes` punya kolom sync-ready dan menulis `change_log` dalam transaksi yang sama, frontend memakai Svelte 5 runes dan Tailwind 4, dan kamus en/id paritasnya 99 = 99 key.

Masalah intinya:
1. Hak akses dan filter visibilitas ditentukan oleh frontend (`caller_profile_id`, `is_owner`), bukan oleh core. Halaman Notes bahkan mengirim `isOwner = true`, sehingga catatan `private` milik siapa pun selalu terlihat.
2. Banyak fitur hanya ada di UI: 25 nama command yang dipanggil frontend tidak terdaftar di Rust, ditambah 4 nama parameter dan 3 tipe data yang tidak cocok. Ganti master password, backup/restore, kirim/abaikan crash report, Pro, dan tampilan jawaban AI diprediksi gagal saat runtime.
3. Lock 2 level (PIN), recovery code, migration runner, guard test konvensi, dan plain-header test belum ada.
4. Generator `new-app` tidak memenuhi FR-12: skemanya berbeda dari PRD, `--with-sample` diabaikan, tidak membuat kunci minisign baru, dan di mesin dev ikut menyalin ±1,7 GB artefak CATerm.
5. `Notes/evidence/` kosong dan `Notes/task.md` tidak ada. Klaim "verified/tests green" di pesan commit S3–S15 tidak punya bukti, dan tag `v0.1.0` tidak ada.

---

## 1. 📊 Matriks Kepatuhan PRD

### Area 1 — Purge SSH/SFTP/FTP/terminal/WebDAV/@xterm

| # | Requirement (sumber) | Status | Bukti |
| --- | --- | --- | --- |
| 1.1 | Crate SSH/SFTP/FTP/PTY/WebDAV/S3 hilang dari dependency (PRD §4, S2.2) | PASS | `Cargo.lock` (629 paket): 0 dari `ssh2`, `libssh2-sys`, `russh*`, `suppaftp`, `portable-pty`, `reqwest_dav`, `rusty-s3`, `aws-sdk-s3`. Cargo.toml keempat crate juga bersih. |
| 1.2 | Modul Rust `ssh`, `sftp`, `scp`, `ftp`, `tunnels`, `vfs`, `s3`, `webdav`, `local_fs`, `monitor`, `keys`, `groups`, `investigations`, `snippets`, `store` dihapus (S2.1) | PASS | `crates/caf-core/src/lib.rs:24-37` hanya berisi ai, audit, backup, crash, db, error, feedback, notes, paths, prefs, pro, profiles, secret, vault. |
| 1.3 | Command Tauri SSH dihapus dari handler, `build.rs`, dan ACL (S2.1) | PASS | `crates/caf-app/src/lib.rs:72-98` (25 command), `build.rs:9-43`, `capabilities/default.json:6-39`: tidak ada ssh/sftp/tunnel. |
| 1.4 | Paket `@xterm/*` dihapus (S2.3) | PASS | Tidak ada di `frontend/package.json`, 0 entri `node_modules/@xterm` di `package-lock.json`, folder `frontend/node_modules/@xterm` tidak ada. |
| 1.5 | Nol sisa kode/teks SSH-terminal (Success criterion 4, S2.4–S2.5) | FAIL | Lihat temuan S-4: 88 file permission basi (`ssh_connect.toml`, `sftp_upload.toml`, `start_tunnel.toml`, …), `frontend/src/lib/nav.ts` (`openSession` → `/session`), `shortcuts.ts:25-33,72-145` (xterm/terminal/sftp), `theme.svelte.ts:56` (`terminalTheme`), `OsIcon.svelte` (505 baris, tak terpakai), `prefs.rs:21-24`, `snap/snapcraft.yaml`, `flatpak/com.fathforce.caterm.*`, `docker-compose.test.yml` (openssh-server). |
| 1.6 | Dependency yatim dibersihkan (S2.2) | PARTIAL | caf-core: `notify`, `md-5`, `hmac`, `ed25519-dalek`, `machine-uid`, `tokio`, `async-trait`, `once_cell`, `rand`, `secrecy` tidak dipakai sama sekali di `src/`. caf-app: `async-trait`, `parking_lot`. Frontend: 12 paket `@codemirror/*`, `autoprefixer`, `postcss` tidak dipakai. |
| 1.7 | Rename `caterm` → `caf` (S1.2: grep hanya boleh menyisakan NOTICE/credits) | FAIL | Alias `CatermError` dipakai di semua modul core (`error.rs:118`, `lib.rs:39`); `release.yml` memakai `CATERM_PRO_LICENSE_PUBKEY_V1`; `build/bin/caterm`, `assets/logo-caterm (1–4).png`, `flatpak/com.fathforce.caterm.*` masih ter-track. |

### Area 2 — Identitas app (`app.toml`) dan codegen (`caf-xtask`)

| # | Requirement (sumber) | Status | Bukti |
| --- | --- | --- | --- |
| 2.1 | Skema `app.toml` sesuai FR-1 | FAIL | Lihat tabel perbandingan di temuan T-6. Nama field berbeda, `[endpoints]` dan `[profiles]` tidak ada, label menu tidak bilingual. Contoh `app.toml` di PRD FR-1 tidak akan bisa di-parse (*prediksi*: `missing field 'bundle_id'`). |
| 2.2 | Metadata identitas, window, security, modules, roles, menus tersedia (permintaan audit) | PASS | `app.toml:1-40` punya `[app]`, `[window]`, `[security]`, `[modules]`, `[roles]`, `[[menus]]`. Catatan: `[window]` dan `[security]` tidak ada di PRD FR-1. |
| 2.3 | Validasi: key tak dikenal ditolak, key menu unik dan URL-safe, accent valid, minimal 1 menu (FR-1, S3.1) | FAIL | `crates/caf-xtask/src/main.rs:36-104`: tidak ada `#[serde(deny_unknown_fields)]`. Satu-satunya validasi adalah `menus.is_empty()` (baris 100-102). |
| 2.4 | Setiap modul bisa dinyalakan/dimatikan dari `app.toml` (FR-1, PRD §3) | FAIL | `APP_CONFIG` hanya dipakai untuk `.name` (`+layout.svelte:80,83,89`). Tombol "Ask AI" selalu tampil (`+layout.svelte:198-206`), UI Pro selalu ada, tidak ada feature flag di Rust. |
| 2.5 | Codegen `app.ts` + `navItems.generated.ts` berheader "generated" dan reproducible (S3.3) | PASS | `main.rs:114-115,177` (header), output deterministik (tanpa timestamp). Uji jalankan-dua-kali: BELUM DIVERIFIKASI. |
| 2.6 | Konstanta prefix error untuk Rust di-generate (S3.3) | FAIL | `error.rs:22,24` meng-hard-code `concat!("CAF-", …)`. |
| 2.7 | `new-app` mengganti nama crate, package, identifier, string produk, data dir, storage key, prefix lewat tabel eksplisit (FR-12, S10.1) | FAIL | `main.rs:277-281` hanya rantai `.replace()`. Nama crate `caf-*`, package `caf-frontend`, dan storage key tidak diganti. Karena urutannya, `bundle_id` tidak pernah diterapkan (lihat T-6). |
| 2.8 | `--with-sample` mengatur ada/tidaknya slice Notes (S10.3) | FAIL | Parameter `_with_sample` diabaikan (`main.rs:217`). |
| 2.9 | Menolak `--out` yang tidak kosong; `--dry-run` mencetak rencana (S10.5) | PARTIAL | Penolakan ada (`main.rs:226-228`). Dry-run hanya mencetak "No files written" (`230-233`). |
| 2.10 | Pasangan kunci minisign baru per app, hanya public key yang ditulis (FR-12, S10.4) | FAIL | Crate `minisign` ada di `crates/caf-xtask/Cargo.toml:16` tetapi tidak dipakai. Pubkey updater framework (`tauri.conf.json:49`) ikut tersalin ke setiap app. |
| 2.11 | Placeholder route per menu, key i18n menu, entri Settings, nama CI, stub `Notes/prd.md` + `Notes/task.md`, `caframework_version` (S10.2) | FAIL | Tidak ada di `main.rs`. |
| 2.12 | Hasil `new-app` bersih dan lulus verifikasi tanpa edit manual (Success criterion 1) | FAIL | `main.rs:254-262`: `starts_with(".git")` ikut membuang `.github/` dan `.gitignore`. `dist/` (788 MB paket CATerm), `crates/caf-app/gen/` (940 MB proyek Android CATerm, termasuk APK 239 MB), dan `build/bin/caterm` ikut tersalin. Compile: BELUM DIVERIFIKASI. |
| 2.13 | Unit test skema dan E2E generator (S3.1, S10.6) | FAIL | `crates/caf-xtask` tidak punya satu pun `#[test]` dan tidak punya folder `tests/`. |

### Area 3 — Keamanan, vault, multi-profil

| # | Requirement (sumber) | Status | Bukti |
| --- | --- | --- | --- |
| 3.1 | Isolasi data dir `~/.local/share/caframework` | PASS | `paths.rs:62-70` (`BaseDirs::data_dir().join("caframework")`), `paths.rs:73-75` (`caframework.db`), override `CAF_DATA_DIR`. Test memakai data dir sementara (`lib.rs:43-99`). Terpisah dari data dir CATerm. |
| 3.2 | SQLCipher + kunci dari master password via Argon2id (FR-4) | PASS | `caf-core/Cargo.toml:44,47` (`bundled-sqlcipher-vendored-openssl`), `vault.rs:16-18,95-107` (Argon2id m=64 MiB, t=3, p=4), `db.rs:42-57` (`PRAGMA key` raw hex), canary AES-GCM `vault.rs:124-134`. |
| 3.3 | Tidak ada kunci plaintext di disk; vault terkunci = akses data ditolak (FR-4, NFR Security) | FAIL | `vault.rs:34-42` dan `206-221`: saat terkunci, core membuat/membaca `vault.key` berisi kunci SQLCipher plaintext. Lihat T-1. |
| 3.4 | Recovery code 24 kata, DEK dibungkus dua kali (FR-4, S6.1b) | FAIL | Tidak ada (grep `recovery`, `mnemonic`, `bip39` = 0 hasil). Kunci DB = Argon2(master) langsung, tanpa DEK. |
| 3.5 | Migration runner dengan tabel `schema_version` (FR-4, S5.2) | FAIL | `schema_version` dibuat tetapi tidak pernah dibaca/ditulis. `db.rs:69-132` hanya `CREATE TABLE IF NOT EXISTS`. |
| 3.6 | Plain-header guard test (S5.6) | FAIL | grep `"SQLite format 3"` di `crates/` = 0. |
| 3.7 | Hash PIN Argon2id disimpan di dalam vault (FR-6) | PASS | `profiles.rs:94-111` (format PHC, salt acak), kolom `pin_hash` di tabel terenkripsi, `#[serde(skip_serializing)]` (`profiles.rs:19`). |
| 3.8 | Lock 2 level di UI: vault → pemilih profil → PIN pad (FR-6, S6.7) | FAIL | `LockScreen.svelte` hanya master password. `verifyPin` (`api/profiles.ts:32`) tidak dipanggil di komponen mana pun. |
| 3.9 | Batas percobaan dan back-off untuk PIN/master password di core (S6.3) | FAIL | `profiles.rs:181-204` tanpa batas. Lockout master password hanya di localStorage (`LockScreen.svelte:30-84`). `VaultError::LockoutActive` tidak pernah dipakai. |
| 3.10 | Dua timer auto-lock (S6.4) | FAIL | Tidak ada di core maupun frontend. |
| 3.11 | Capability check (aksi khusus owner) di core (FR-6, S6.5) | FAIL | `commands.rs:88-121` menerima `caller_profile_id`/`is_owner` dari frontend. `reset_vault`, `change_master_password`, `export_encrypted_backup` tanpa cek peran. `AuthError::*` tidak pernah dipakai. Lihat C-1. |
| 3.12 | Filter visibilitas di core tidak bisa di-bypass (FR-6, S6.6) | FAIL | `notes.rs:71` (`… \|\| is_owner`), `+page.svelte:38` (`isOwner = true`), update/delete tanpa cek pemilik (`notes.rs:90-107`, `178-223`). |
| 3.13 | Zeroize/secrecy saat vault dikunci (NFR Security) | PARTIAL | `vault.rs:49-54` men-zeroize *salinan* array (tipe `Copy`); byte kunci tertinggal di static. String hex kunci (`vault.rs:37`, `db.rs:21,44,54`) tidak di-zeroize. Crate `secrecy` tidak dipakai. Lihat S-1. |
| 3.14 | Indeks profil di luar vault (nama + avatar) dan opsi sembunyikan nama (S6.2) | PARTIAL | Hanya profil tunggal gaya CATerm di localStorage (`stores/profile.svelte.ts:10`), tidak terhubung dengan tabel `profiles`. Opsi sembunyikan nama tidak ada. |

### Area 4 — Konvensi data sync-ready

| # | Requirement (sumber) | Status | Bukti |
| --- | --- | --- | --- |
| 4.1 | Semua tabel data user punya `id`, `rev`, `created_at`, `updated_at`, `deleted_at`, `origin_device_id`, `owner_profile_id`, `visibility` (FR-5) | PARTIAL | `profiles` dan `notes` lengkap (`db.rs:82-123`). `app_kv` tidak punya kolom tersebut dan tidak dikecualikan secara eksplisit di mana pun. |
| 4.2 | `id` UUIDv7 | PASS | `profiles.rs:133`, `notes.rs:110` (`Uuid::now_v7()`). |
| 4.3 | `created_at`/`updated_at` dalam unix ms (FR-5) | FAIL | Kolom `TEXT`. Default `datetime('now')` (`db.rs:89-90,117-118`) bercampur dengan RFC 3339 dari kode (`notes.rs:85`, `profiles.rs:91`): dua format dalam satu kolom. |
| 4.4 | `rev` naik setiap perubahan | PARTIAL | notes ✓ (`notes.rs:100,195`), update profil ✓ (`profiles.rs:122`), tetapi `delete_profile` tidak menaikkan `rev` (`profiles.rs:255-258`). |
| 4.5 | Hapus = soft delete | PASS | `notes.rs:196-200`, `profiles.rs:255-258`. |
| 4.6 | Setiap write menambah baris `change_log` dalam transaksi yang sama | PARTIAL | notes ✓ (`notes.rs:143-158`, `202-217`). profiles ✗: `save_profile` (`113-157`) dan `delete_profile` (`248-270`) tidak menulis `change_log`. |
| 4.7 | Trigger `change_log` otomatis (permintaan audit) | FAIL | Tidak ada `CREATE TRIGGER` di repo. Pencatatan manual hanya di `notes.rs`. |
| 4.8 | Guard test konvensi via `sqlite_master` (S5.4) | FAIL | Tidak ada test. |
| 4.9 | Test rev, log, dan rollback (S5.3) | PARTIAL | `notes.rs:229-277` hanya memeriksa `rev` dan hasil list. Tidak memeriksa isi `change_log`, tidak ada test rollback. |
| 4.10 | Device id dibuat sekali dan disimpan (S5.5) | PASS | `paths.rs:81-104` (UUIDv4 di `device_id.txt`). |

### Area 5 — Frontend dan design system

| # | Requirement (sumber) | Status | Bukti |
| --- | --- | --- | --- |
| 5.1 | Svelte 5 runes, store di `*.svelte.ts` | PASS | 100× `$state`, 13× `$derived`, 18× `$props`; 0× `$:`, `on:`, `createEventDispatcher`, `<slot>`, `svelte/store`. Pengecualian: `Logo.svelte:2-4` masih `export let`. |
| 5.2 | Tailwind 4 (`@tailwindcss/vite`, `@custom-variant dark`, token) | PASS | `app.css:1,15,25-40` (token sama persis dengan kontrak `prompt-dev.md` §4), `vite.config.js`, tanpa `tailwind.config`. |
| 5.3 | i18n ID/EN bertipe dan lengkap (FR-10) | PARTIAL | `en.ts` 99 key = `id.ts` 99 key, `id: Dictionary` ✓. Tetapi **134 dari 159 key yang dipakai kode tidak ada di kamus** (lock 37, pro 25, feedback 22, crash 13, about 12, profileMenu 10, updater 6, lainnya 9). `t()` mengembalikan key mentah, jadi layar kunci menampilkan teks seperti `lock.welcomeBack`. `t(key: string)` tidak type-safe. |
| 5.4 | Tema dark/light dinamis | FAIL | Shell (`+layout.svelte`), Notes, Settings, `PageHeader`, Empty/Loading/ErrorState, `AiChatPanel`, `CommandPalette`: 0 varian `dark:` dengan kelas gelap hard-coded. Hanya `LockScreen` (59). |
| 5.5 | Aksen dinamis dari `app.toml`, map kelas literal 10 aksen | FAIL | `APP_CONFIG.accentColor` tidak dipakai, 62 kelas `indigo-*` hard-coded. `PageHeader` tanpa prop ikon/aksen. |
| 5.6 | State components Loading/Empty/Error/BackendUnavailable + confirm modal (S4.7) | PARTIAL | `LoadingState`, `EmptyState`, `ErrorState` ada (pakai `$props`). `ErrorState` tidak dipakai di halaman mana pun, `BackendUnavailable` tidak ada, Notes menampilkan EmptyState saat error. Confirm modal ✓ (`confirmModal`). |
| 5.7 | `PageContainer` di setiap halaman | FAIL | 0 pemakaian. |
| 5.8 | Title bar custom + drag region + kontrol jendela (FR-2) | FAIL | Hanya ada di `LockScreen`. Setelah unlock, jendela `decorations: false` (`tauri.conf.json:18`) tidak punya title bar, tombol minimize/maximize/close, atau area seret. |
| 5.9 | Sidebar dari nav generated, dengan grup dan filter role; drawer responsif (FR-2, S4.3-4.4) | PARTIAL | Sumber `GENERATED_NAV_ITEMS` ✓. Tanpa grup, filter role, label i18n. 0 breakpoint `sm:`/`md:` di layout. |
| 5.10 | Settings registry (FR-3, S4.5) | FAIL | Tab hard-coded (`settings/+page.svelte:14`). Section Appearance, Language, Updates, Privacy, Shortcuts, Pro tidak ada. |
| 5.11 | Helper format angka, mata uang, tanggal ID/EN (FR-10, S4.8) | FAIL | Hanya `formatUsd` (`pro/pricing.ts:22`). |
| 5.12 | Wiring UI → command nyata ("Wiring, not UI") | FAIL | 25 command tidak terdaftar + 4 parameter + 3 tipe tidak cocok. Lihat T-4. |

### Area 6 — AI assistant dan privacy redaction

| # | Requirement (sumber) | Status | Bukti |
| --- | --- | --- | --- |
| 6.1 | BYO OpenAI-compatible + Ollama di core (FR-7) | PASS | `ai.rs:140-229`. Panggilan nyata: BELUM DIVERIFIKASI. |
| 6.2 | Mode Off / BYO / Hosted (hanya bila Pro aktif) | FAIL | Tidak ada mode Off. `hosted` dan `anthropic` jatuh ke jalur OpenAI (`ai.rs:147-154`), sehingga key pengguna dapat terkirim ke `api.openai.com`. |
| 6.3 | API key di vault, tidak di localStorage | PASS | Disimpan di `app_kv` di DB terenkripsi (`ai.rs:79-91`). Tidak ada `api_key` di localStorage. Catatan: `get_ai_settings` mengembalikan key penuh ke webview. |
| 6.4 | Scrubber regex email → `[REDACTED_EMAIL]`, kartu/rekening → `[REDACTED_ACCOUNT]` | PARTIAL | Ada, beserta 1 test (`ai.rs:94-114`, `236-245`). Hanya berlaku untuk `context`, bukan `prompt`. Pola kartu hanya 16 digit 4-4-4-4. Tidak menangkap nomor telepon, NPWP, nama. Ada bug kelas karakter di regex email. `.unwrap()` melanggar `deny(clippy::unwrap_used)`. |
| 6.5 | Privacy level (anonymised summary default, detail dengan consent per percakapan, detail penuh untuk endpoint lokal) + registry context provider (FR-7, S7.2) | FAIL | Hanya boolean `privacy_redaction_enabled`. UI tidak pernah mengirim context. |
| 6.6 | Guardrail produk tetap, disusun sebelum instruksi user (S7.3) | PARTIAL | Urutan benar (`ai.rs:156,190-196`), tetapi teks hard-coded untuk CAFramework dan tidak bisa diatur per produk. UI menampilkan toggle "Enforce System Guardrails" yang tidak berfungsi dan bertentangan dengan "non-user-editable". |
| 6.7 | Bisa diatur: base URL, model, temperature, bahasa, tone, custom instructions | PARTIAL | 4 dari 6 (bahasa, tone, custom instructions tidak ada). Temperature tidak dikirim ke Ollama. |
| 6.8 | Error state `AI_NOT_CONFIGURED`, quota, jaringan (S7.4) | FAIL | Tanpa API key, core mengembalikan respons "sukses" palsu (`ai.rs:129-138`). `AiError::ApiKeyMissing` tidak dipakai. |
| 6.9 | UI chat menampilkan jawaban | FAIL | `AiChatPanel.svelte:25-26,64` memperlakukan objek `AiChatResponse` sebagai string, sehingga yang tampil `[object Object]`. |
| 6.10 | Scrubber PII crash report (FR-8) | PARTIAL | Private key, API key `sk-`/`gsk_`, pasangan key=value rahasia, IPv4, home dir, username (`crash.rs:187-261`) ✓. Email dan nomor kartu tidak di-scrub. |

### Area 7 — Test suite, bukti, dan artefak rilis

| # | Requirement (sumber) | Status | Bukti |
| --- | --- | --- | --- |
| 7.1 | Cakupan test wajib PRD (guard, plain-header, auth/visibilitas, backup round-trip, migrasi, skema xtask, E2E) | FAIL | Hanya 30 test: 24 unit caf-core, 4 arch, 2 error-codes. 0 test di caf-app, caf-cli, caf-xtask. Semua test wajib di kolom kiri tidak ada. |
| 7.2 | `cargo clippy --workspace --all-targets -- -D warnings` | FAIL (*prediksi*) | `ai.rs:99,107` memanggil `.unwrap()` di kode non-test di bawah `#![deny(clippy::unwrap_used)]` (`lib.rs:4-5`). |
| 7.3 | `cargo test`, `fmt --check`, `deny check`, `npm ci && npm run check && npm run build`, `cargo check -p caf-app` | BELUM DIVERIFIKASI | Tidak dijalankan (audit statis). Perintah di §5. |
| 7.4 | Bukti per tahap di `Notes/evidence/S<n>.md`, checklist di `Notes/task.md` | FAIL | `Notes/evidence/` kosong, `Notes/task.md` tidak ada. |
| 7.5 | Release binary `caf-app` | PARTIAL | `target/release/caf-app` ada (13.747.264 byte, 2 Okt 2026 06:23 WIB). Peluncuran BELUM DIVERIFIKASI. |
| 7.6 | Desktop launcher | FAIL (repo) | Commit `7ec9e64` mengklaim "desktop launcher" tetapi hanya mengubah 7 file docs/CI. Satu-satunya `.desktop` di repo adalah `flatpak/com.fathforce.caterm.desktop` dengan `Comment=Zero-Knowledge SSH Terminal & SFTP Client`. Di sistem: BELUM DIVERIFIKASI. |
| 7.7 | Tag `v0.1.0` (S15.1) | FAIL | `git tag` kosong; versi masih `0.1.0-dev` (`tauri.conf.json:4`), padahal pesan commit menyebut "v0.1.0 release build". |

### Ringkasan per requirement PRD

| Requirement | Status | Butir pendukung |
| --- | --- | --- |
| FR-1 App identity (`app.toml`) | FAIL | 2.1, 2.3, 2.4 |
| FR-2 Shell, navigasi, design system | FAIL | 5.4, 5.5, 5.8, 5.9 |
| FR-3 Settings registry | FAIL | 5.10 |
| FR-4 Vault dan storage | PARTIAL | 3.2 ✓; 3.3, 3.4, 3.5, 3.6 ✗ |
| FR-5 Konvensi sync-ready | PARTIAL | 4.1–4.10 |
| FR-6 Multi-profil, PIN, role, ownership | FAIL | 3.8–3.12 |
| FR-7 AI assistant | PARTIAL | 6.1, 6.3 ✓; 6.2, 6.5, 6.8, 6.9 ✗ |
| FR-8 Pro, feedback, updater, crash | PARTIAL | Crash hook/0600/scrubber ✓; Pro hanya stub + UI (S-6); submit/dismiss crash rusak (T-4) |
| FR-9 Backup dan restore | FAIL | T-5 |
| FR-10 i18n dan formatting | PARTIAL | 5.3, 5.11 |
| FR-11 Menu contoh Notes | PARTIAL | CRUD + change_log ✓; visibilitas bisa di-bypass, profil hard-coded, tanpa import/export JSON |
| FR-12 Generator `new-app` | FAIL | 2.7–2.13 |
| FR-13 Android | BELUM DIAUDIT | Di luar 7 area; dua risiko di S-9 |
| FR-14 Kebersihan open-source | FAIL | S-5 |
| NFR Security | FAIL | C-1, T-1, S-1 |
| NFR Privacy | PARTIAL | Crash opt-in dan dump lokal ✓; AI tanpa anonymised summary (6.5) |
| NFR Quality | FAIL | 7.1, 7.2 |

---

## 2. 🔍 Temuan Kritis / Gaps

Skala keparahan mengikuti Appendix A `prompt-dev.md`: Kritis / Tinggi / Sedang / Rendah.

### C-1 · KRITIS — Identitas dan hak akses ditentukan frontend; core tidak melakukan capability check

- **Lokasi:** `crates/caf-app/src/commands.rs:88-121` (`save_profile`, `list_notes`, `save_note`, `delete_note` menerima `caller_profile_id` dan `is_owner` dari JS), `commands.rs:72-80,167-175` (`change_master_password`, `reset_vault`, export/import backup tanpa cek peran), `crates/caf-core/src/notes.rs:71`, `notes.rs:90-107`, `notes.rs:178-223`, `crates/caf-core/src/profiles.rs:82-179`.
- **Bukti:**
  - Filter list: `if note.visibility == "shared" || note.owner_profile_id == caller_profile_id || is_owner` (`notes.rs:71`), dengan `is_owner` dikirim oleh frontend.
  - UI Notes: `currentProfileId = '018e0000-0000-7000-8000-000000000001'` (`frontend/src/routes/+page.svelte:21`) dan `listNotes(currentProfileId, true)` (`:38`), sehingga semua catatan, termasuk `private`, selalu tampil.
  - Settings membuat profil dengan caller `'00000000-0000-0000-0000-000000000000'` (`settings/+page.svelte:128`), dan role apa pun (termasuk `owner`) diterima tanpa validasi.
  - Update profil boleh mengganti PIN profil lain (`profiles.rs:113-131`).
  - Update dan hapus catatan tidak membandingkan `owner_profile_id` dengan pemanggil.
  - `export_backup` memanggil `list_notes("backup-root", true)` (`backup.rs:26`), jadi semua catatan private ikut diekspor tanpa cek owner.
  - Tidak satu pun varian `AuthError::{Unauthorized, InvalidPin, PermissionDenied}` dipakai di kode.
- **Dampak:** FR-6 dan success criterion "profil tidak bisa membaca baris `private` profil lain lewat command apa pun" gagal total. Di alur UI normal pun catatan private terlihat oleh semua orang. Lewat command yang sudah terdaftar, siapa pun di perangkat bisa membuat profil `owner`, mengganti PIN orang lain, me-reset vault, atau mengekspor semua data. Untuk produk keluarga seperti CACash Family dengan profil anak, ini inti model keamanannya.
- **Perbaikan:** lihat P0-1 dan P0-2 di §3.

### T-1 · TINGGI — Saat vault terkunci, core memakai kunci SQLCipher plaintext `vault.key`

- **Lokasi:** `crates/caf-core/src/vault.rs:34-42` (`ensure_unlocked_key`), `vault.rs:206-221` (`load_or_create_local_key` menulis 64 karakter hex ke `<data_dir>/vault.key` tanpa permission 0600), `db.rs:135-139`.
- **Bukti:** jika `ACTIVE_VAULT_KEY` kosong, `ensure_unlocked_key()` tidak mengembalikan `VaultError::Locked` tetapi membuat/membaca `vault.key`. Ketiga test yang menyentuh DB (`profiles.rs:279`, `notes.rs:232`, `ai.rs:250`) berjalan di jalur ini tanpa master password. Artinya test hijau tidak membuktikan alur master password.
- **Dampak:**
  - Core tidak punya state "terkunci". Command data tidak pernah mengembalikan `CAF-VAULT-001`; yang muncul adalah error generik `invalid vault key or corrupted database` (`CAF-DB-000`), dan file `vault.key` tertinggal.
  - Jika ada akses DB sebelum setup pertama (misalnya `caframeworkctl profile add`, atau pemilih profil di layar kunci yang diminta PRD), database dibuat dengan kunci yang tersimpan plaintext di sebelahnya.
  - Setelah itu setup master password gagal membuka DB tersebut dan memaksa reset.
  - Di alur GUI saat ini risikonya masih laten, tetapi setiap produk yang dibuat dari template mewarisinya.
- **Perbaikan:** P0-3.

### T-2 · TINGGI — Lock level 2 (PIN profil) belum ada; tanpa batas percobaan dan auto-lock

- **Lokasi:** `frontend/src/lib/components/LockScreen.svelte` (hanya master password; `verifyPin` tidak diimpor), `profiles.rs:181-204`, `LockScreen.svelte:30-84`.
- **Bukti:**
  - `verify_pin` hanya mengembalikan `bool` tanpa membuat sesi, dan mengembalikan `true` bila PIN belum di-set (`profiles.rs:191-197`), termasuk untuk role yang wajib PIN.
  - Lockout master password hanya disimpan di localStorage dan bisa dihapus.
  - Profil yang tampil di UI adalah profil tunggal localStorage gaya CATerm (`stores/profile.svelte.ts`), tidak terhubung dengan tabel `profiles`.
  - Tidak ada timer auto-lock.
- **Dampak:** FR-6 dan S6.3, S6.4, S6.7, S6.8 gagal. Keamanan level profil tidak ada.

### T-3 · TINGGI — Recovery code 24 kata dan DEK wrapping tidak ada

- **Lokasi:** `vault.rs` seluruhnya. Kunci DB = Argon2(master) langsung (`vault.rs:143-149`); ganti password melakukan `PRAGMA rekey` seluruh DB (`vault.rs:189-195`).
- **Dampak:** FR-4 dan D9 gagal. Lupa master password berarti kehilangan semua data; satu-satunya jalan adalah reset. Ganti password juga tidak atomik: DB di-rekey dulu, baru canary ditulis (`vault.rs:191` lalu `194-195`). Jika penulisan canary gagal, vault tidak bisa dibuka. Salt lama dipakai ulang.

### T-4 · TINGGI — Banyak fitur hanya UI: command tidak terdaftar dan parameter/tipe tidak cocok

- **Bukti:** set command yang dipanggil `invoke()` di frontend dibandingkan dengan `generate_handler!` (`lib.rs:72-98`):
  ```
  Registered di generate_handler!: 25
  Dipanggil frontend: 50
  DIPANGGIL FRONTEND tapi TIDAK TERDAFTAR:
  export_vault_backup import_vault_backup is_vault_unlocked open_external_url pro_account
  pro_commit_pending pro_forgot_password pro_login pro_logout pro_register
  pro_resend_verification pro_revoke_device pro_server_available pro_start_trial pro_status
  pro_sync pro_team pro_team_accept pro_team_cancel_invite pro_team_decline pro_team_invite
  pro_team_leave pro_team_remove_member save_performance_prefs submit_crash_report
  ```
- **Parameter tidak cocok.** Tauri 2 mengharapkan argumen dalam camelCase dari nama argumen Rust.

  | Command | Frontend mengirim | Rust mengharapkan |
  | --- | --- | --- |
  | `change_master_password` | `{ currentPassword, newPassword }` (`api/vault.ts:26`) | `old`, `new` (`commands.rs:73`) |
  | `export_encrypted_backup` | `{ passphrase }` (`api/backup.ts:4`) | `password`; hasilnya `Vec<u8>`, bukan string |
  | `import_encrypted_backup` | `{ encryptedB64, passphrase }` (`api/backup.ts:8`) | `data: Vec<u8>`, `password` |
  | `dismiss_crash_report` | `{ reportId, neverAgain }` (`api/crash.ts:34`) | `id`; `never_again` di-hard-code `false` (`commands.rs:163`) |

- **Tipe tidak cocok:**
  - `ai_chat` mengembalikan objek `{ message, redactions_applied, tokens_used }`, sedangkan TS menganggapnya `string`, sehingga tampil `[object Object]`.
  - `AiSettings` di TS (`privacy_mode`, `guardrails_enabled`) tidak ada di Rust (`privacy_redaction_enabled`, `temperature`), sehingga toggle privasi di UI tidak berefek.
  - Profil: TS `avatar_url` vs Rust `avatar`, sehingga avatar tidak tersimpan. UI membaca `pin_hash` yang sengaja tidak diserialisasi, sehingga selalu tampil "No PIN" (`settings/+page.svelte:309`); seharusnya memakai `has_pin`.
- **Dampak (prediksi runtime):** ganti master password, backup/restore (Settings memakai `api/prefs.ts` → `export_vault_backup` yang tidak ada), kirim/abaikan crash report (modal akan muncul lagi setiap peluncuran), seluruh UI Pro, simpan preferensi performa, buka URL eksternal, dan tampilan jawaban AI semuanya gagal. Arch test `tauri_command_registry_build_script_and_acl_agree` hanya mencocokkan `lib.rs`, `build.rs`, dan ACL, tidak memeriksa pemanggilan dari frontend.

### T-5 · TINGGI — Backup/restore tidak bisa memulihkan data dan kriptografinya lemah

- **Lokasi:** `crates/caf-core/src/backup.rs`.
- **Bukti dan dampak:**
  - Restore memanggil `save_profile`/`save_note` dengan `id: Some(...)` (`backup.rs:76-101`). Jalur itu adalah UPDATE yang mensyaratkan baris sudah ada (`profiles.rs:113-120`, `notes.rs:90-98`). Di vault kosong (skenario backup → wipe → restore), **tidak ada yang dipulihkan**, error ditelan `let _ =`, dan fungsi tetap mengembalikan `Ok(())`.
  - PIN tidak ikut (`pin: None`), tombstone dan `change_log` tidak diekspor.
  - Salt Argon2 tetap untuk semua pengguna dan semua backup: `b"caframework.vault.backup.salt.2026"` (`backup.rs:47,65`).
  - Tidak ada versi format yang divalidasi: `version` = versi crate, dan tidak diperiksa saat import.
  - Tidak ada backup terjadwal (S5.7b) dan tidak ada test round-trip (S5.7).

### T-6 · TINGGI — Generator `new-app` dan skema `app.toml` tidak memenuhi FR-1 dan FR-12

- **Skema** (`crates/caf-xtask/src/main.rs:36-95` dibandingkan PRD FR-1):

  | Bagian | PRD FR-1 | Implementasi |
  | --- | --- | --- |
  | `[app]` | `identifier`, `accent`, `code`, `default_locale`, `locales`, `platforms`, `caframework_version` | `bundle_id`, `accent_color`, `error_prefix`, `version`, `description`; empat field PRD tidak ada |
  | `[modules]` | `profiles`, `ai`, `pro`, `feedback`, `updater`, `crash`, `backup`, `support_page` | `*_enabled` (6 switch); `profiles`, `feedback`, `support_page` tidak ada |
  | `[profiles]` | `roles`, `owner_role`, `pin_required_for`, `child_age_stages` | diganti `[roles] default_role`, `allowed_roles` |
  | `[endpoints]` | `gcc_base_url`, `updater_manifest_url` | tidak ada (endpoint di-hard-code, lihat S-6) |
  | `[[menus]]` | `key`, `label_en`, `label_id`, `icon` (path SVG), `accent`, `group`, `roles` | `key`, `label`, `href`, `icon` (nama), `description` |

  Semua field implementasi wajib (tanpa `Option`/default), jadi `app.toml` yang ditulis sesuai PRD, termasuk `app.toml` CACash di S15.2, akan ditolak parser (*prediksi*).
- **Validasi:** hanya `menus.is_empty()`. Tidak ada `deny_unknown_fields`, cek keunikan atau URL-safe key, cek aksen, atau cek role.
- **Mesin rename** (`main.rs:277-281`):
  - Nama crate (`caf-core`, `caf-app`, `caf-cli`, `caf-xtask`), `caf-frontend`, dan storage key tidak diganti.
  - Karena `"caframework"` diganti lebih dulu, string `"com.fathforce.caframework"` sudah berubah sebelum giliran penggantiannya, sehingga `bundle_id` tidak pernah diterapkan. Identifier selalu menjadi `com.fathforce.<slug>`.
  - File `.js`, `.css`, `.html`, `.yml`, `.sh`, `.desktop`, `.xml`, dan `.lock` disalin apa adanya.
- **Penyalinan** (`main.rs:248-287`):
  - `rel_str.starts_with(".git")` ikut membuang `.github/` (CI), `.gitignore`, dan `.gitattributes`.
  - Tidak ada pengecualian untuk `dist/` (788 MB paket CATerm), `crates/caf-app/gen/` (940 MB proyek Android CATerm `com/fathforce/caterm`, termasuk `app-universal-debug.apk` 239 MB dan 3× `libcaterm_app_lib.so` 231 MB), `build/bin/caterm`, maupun dokumen CATerm di root.
  - Jika `--out` berada di dalam repo, WalkDir ikut menelusuri folder tujuan.
- **Fitur yang hilang:** `--with-sample` diabaikan; tidak ada pembuatan kunci minisign (semua app berbagi kunci updater framework); tidak ada placeholder route, i18n menu, stub `Notes/*`, `caframework_version`; dry-run tidak mencetak rencana; tidak ada test.
- Pesan commit `4f65b97` ("verified with E2E scaffold test") tidak didukung test apa pun di repo.

### T-7 · TINGGI — Tidak ada migration runner, guard test konvensi, dan plain-header test

- **Lokasi:** `db.rs:69-132` (hanya `CREATE TABLE IF NOT EXISTS`, `schema_version` tidak terpakai). Tidak ada test S5.4 dan S5.6.
- **Dampak:** produk tidak bisa menambah atau mengubah kolom secara aman (perubahan skema berisiko merusak data pengguna), dan tidak ada mekanisme otomatis yang menjamin FR-5 untuk tabel baru.

### T-8 · TINGGI — Shell UI tidak sesuai kontrak dan teks layar kunci rusak

- Setelah unlock tidak ada title bar atau tombol kontrol jendela, sementara jendela `decorations: false`. Pengguna desktop tidak bisa memindah, memperkecil, atau menutup jendela lewat UI (`+layout.svelte:67-232`, `tauri.conf.json:18`).
- 134 key i18n yang dipakai tidak ada di kamus, sehingga layar pertama yang dilihat pengguna (LockScreen, 37 key `lock.*`) menampilkan key mentah.
- Tema terang tidak berfungsi setelah unlock (0 `dark:` di shell, Notes, Settings, komponen state).
- Aksen hard-coded indigo (62 kelas). `PageHeader` tanpa ikon/aksen. `PageContainer` tidak dipakai. Tidak ada drawer responsif. Settings tanpa registry.
- Banyak string Inggris hard-coded: label Settings, toast, "Ask AI", "Lock Vault", default teks di `EmptyState`/`LoadingState`/`ErrorState`.

### T-9 · TINGGI — Klaim penyelesaian tahap tanpa bukti

- `Notes/evidence/` kosong, `Notes/task.md` tidak ada, tag `v0.1.0` tidak ada.
- Pesan commit `1237508`…`7ec9e64` mengklaim "tests pass", "verified with E2E", "v0.1.0 release build", "desktop launcher".
- Sesuai `task.md` ("Tick [x] only when the proof is recorded") dan `prompt-dev.md` §2.2, semua tahap S3–S15 statusnya **BELUM DIVERIFIKASI**. Audit ini juga menemukan bertentangan untuk sebagian klaim (misalnya E2E generator dan launcher).

### T-10 · TINGGI — Privasi AI belum sesuai desain FR-7

- **Lokasi:** `crates/caf-core/src/ai.rs`.
- **Temuan:**
  - Tidak ada privacy level, registry context provider, consent per percakapan, maupun aturan "detail penuh hanya untuk endpoint lokal". Hanya ada boolean redaksi.
  - Provider `anthropic`/`hosted` jatuh ke jalur OpenAI-compatible. Dengan endpoint default `https://api.openai.com/v1` (`ai.rs:45,149`, juga default UI `settings/+page.svelte:31`), API key milik provider lain dikirim sebagai Bearer token ke OpenAI.
  - `prompt` tidak di-scrub (`ai.rs:119-127` hanya memproses `context`).
  - Regex kartu hanya menangkap 16 digit 4-4-4-4. Regex email memakai kelas karakter `[A-Z|a-z]` (karakter `|` ikut terhitung literal). Nama tidak dianonimkan meski dokumentasi fungsi mengklaimnya (`ai.rs:93`).
  - Respons sukses palsu saat belum dikonfigurasi (`ai.rs:129-138`).
  - Tanpa timeout HTTP.
  - `get_ai_settings` mengirim API key penuh ke webview.

### S-1 · SEDANG — Zeroization tidak efektif

`vault.rs:49-54`: `guard.take()` memindahkan salinan `[u8; 32]` (tipe `Copy`), lalu hanya salinan itu yang di-zeroize. Byte kunci asli tetap ada di memori static sampai ditimpa. Kunci juga beredar sebagai `String` hex (`vault.rs:37`, `db.rs:21,44,54` di dalam `format!` PRAGMA) dan master password sebagai `String` argumen command, semuanya tanpa zeroize. Crate `secrecy` dideklarasikan tetapi tidak dipakai.

### S-2 · SEDANG — Clippy `-D warnings` diprediksi gagal

`ai.rs:99` dan `ai.rs:107` memanggil `Regex::new(...).unwrap()` di kode non-test, sementara crate memasang `#![deny(clippy::unwrap_used)]` (`lib.rs:4-5`). CI saat ini juga tidak menjalankan clippy (lihat S-5), jadi kesalahan ini tidak terdeteksi.

### S-3 · SEDANG — Konvensi data dan audit log tidak konsisten

- `profiles` tidak menulis `change_log`, dan `delete_profile` tidak menaikkan `rev`.
- Timestamp memakai dua format teks berbeda (bukan unix ms).
- Nama kolom `change_log` berbeda dari PRD (`entity_type`/`action`/`timestamp` vs `entity`/`op`/`at`).
- `audit::log_event` menulis ke tabel `command_logs` yang **tidak pernah dibuat** (`audit.rs:46-50`, tidak ada di `db.rs`), sehingga semua event audit (VAULT_INIT, VAULT_UNLOCK, VAULT_REKEY, PROFILE_DELETE) gagal diam-diam. Kolomnya masih bernama `host_id`, warisan SSH.

### S-4 · SEDANG — Sisa SSH/terminal di repo

- **Backend:**
  - 88 dari 113 file `crates/caf-app/permissions/autogenerated/*.toml` basi: `ssh_*`, `sftp_*`, `*_tunnel`, `*_host`, `*_key`, `local_*`, `*_remote_*`, `*_watch`, `get_command_logs`, dan lain-lain.
  - `prefs.rs:21-24` (scrollback, inactive session).
  - Komentar SSH di `secret.rs:1-7`, `window.rs:40,79,149`, `crash.rs:171`.
  - `audit.rs` (`host_id`).
- **Frontend:**
  - `lib/nav.ts`, `shortcuts.ts` (grup terminal/sftp/sessions, `isTerminalTarget`, `terminalKeyAction`), `theme.svelte.ts:56` (`terminalTheme`), `commandPalette.svelte.ts` (mode `hosts`).
  - `OsIcon.svelte` (tak terpakai), preset avatar `terminal`, `static/hostinger-logo.jpg` (tak terpakai), `api/performance.ts` (`scrollbackLines`, `inactiveSessionSleep`).
- **Packaging/dokumen:**
  - `snap/snapcraft.yaml` ("SSH Terminal & SFTP Client", plug `ssh-keys`), `flatpak/com.fathforce.caterm.*`, `docker-compose.test.yml` (openssh-server).
  - `task-v2.md`, `ledger-v2.md`, `agent-master-prompt.md`, `caterm-execution-prompts-v2-addendum.md`, `optimization.md` di root.
- **Lokal (tidak ter-track):** `dist/` (paket CATerm), `crates/caf-app/gen/android` (proyek Android CATerm).

### S-5 · SEDANG — Kebersihan open-source (FR-14) dan CI

- `build/bin/caterm` (15,1 MB) ter-track, padahal aturannya tidak boleh ada biner atau file > 1 MB.
- Path absolut mesin dev ada di `docs/android-build-guide.md:34`, `docs/android-signing.md:11,28,34`, dan `caterm-execution-prompts-v2-addendum.md`.
- `ci.yml` (58 baris) hanya menjalankan fmt, test, dan npm. Tidak ada clippy, `cargo deny`, guard biner/ukuran/path absolut, E2E generator, maupun pemeriksaan registry command frontend. Commit terakhir menghapus 156 baris dan menambah 28 baris di `ci.yml` (`git show --numstat HEAD`).
- `CHANGELOG.md`, `docs/security.md`, `docs/backend-contract.md`, `docs/android.md`, dan `docs/porting-framework-fixes.md` tidak ada.

### S-6 · SEDANG — Endpoint privat hard-coded, tanpa `app_id`; Pro hanya stub

- Endpoint hard-coded: `crash.rs:21` dan `feedback.rs:10` (`https://caframework.fathforce.com/api/...`), `tauri.conf.json:51` (manifest updater), `appInfo.ts:7-15` (website, pricing, sponsor).
- Tidak ada `app_id` di request mana pun (FR-8), dan modul tidak bisa dimatikan karena tidak ada `[endpoints]`.
- `pro.rs` hanya 21 baris yang mengembalikan status `free`, dan tidak diekspos sebagai command. Sementara itu UI Pro lengkap (LockScreen punya mode login Pro) dan `app.toml` menulis `pro_licensing_enabled = true`.

### S-7 · SEDANG — CLI menerima rahasia lewat argumen command line

`caf-cli/src/main.rs:46` (`vault unlock <password>`) dan `:60` (`profile add --pin`): rahasia tercatat di shell history dan daftar proses. Ini pitfall yang disebut di `prompt-dev.md` §7. Selain itu `notes list` memakai `list_notes("cli-root", true)`, sehingga semua catatan private ikut terbaca.

### S-8 · SEDANG — Scrubber crash tidak mencakup email dan nomor kartu

`crash.rs:187-199` tidak punya aturan email atau kartu, padahal pesan panic dan backtrace bisa memuat data pengguna.

### S-9 · SEDANG — Risiko Android (di luar 7 area, perlu verifikasi)

- `capabilities/default.json:37` memuat `updater:default`, padahal `desktop.json:4` menyatakan izin updater tidak boleh berada di capability bersama karena plugin tidak terdaftar di mobile. Build Android berisiko gagal.
- `crates/caf-app/gen/android` adalah proyek Android CATerm (package `com/fathforce/caterm`). Folder ini tidak ter-track, tetapi build lokal akan memakai identitas CATerm.

### Rendah

- R-1: mode portable menulis ke `./caframework-data`, relatif terhadap CWD, bukan folder executable (`paths.rs:57`).
- R-2: `vault_salt.bin`, `vault_canary.bin`, `device_id.txt`, `vault.key` ditulis tanpa permission 0600.
- R-3: `+layout.svelte:42` memanggil `is_vault_unlocked` yang tidak ada. Kebetulan aman karena selalu jatuh ke layar kunci, tetapi menyesatkan.
- R-4: `Logo.svelte` masih memakai props gaya Svelte 4.
- R-5: Hampir semua error memakai varian `Generic` (`CAF-*-000`). Frontend tidak bisa membedakan "password salah" dari error lain berdasarkan kode.

---

## 3. 🛠️ Rekomendasi / Langkah Perbaikan Konkret

Disusun bertahap supaya agent tidak kehilangan konteks. Kerjakan satu tahap, tulis bukti di `Notes/evidence/`, lalu lapor dan berhenti di gate.

### P0 — Blocker keamanan (wajib sebelum fitur lain)

1. **Sesi di core.**
   - Tambahkan `SessionState` (Tauri managed state atau state global di core) berisi `active_profile_id` dan `role`. Nilainya hanya boleh di-set oleh `unlock_vault` + `verify_pin` yang sukses.
   - Hapus parameter `caller_profile_id` dan `is_owner` dari semua command.
   - Owner-only di core: membuat/mengubah profil lain dan role, `reset_vault` (dengan token konfirmasi), `change_master_password`, export/import backup. Tolak dengan `CAF-AUTH-001`/`003`.
   - *Bukti selesai:* test yang memanggil setiap command sebagai non-owner dan mendapat `CAF-AUTH-*`.
2. **Helper visibilitas di core**, dipakai semua modul.
   - Filter di SQL: `deleted_at IS NULL AND (visibility = 'shared' OR owner_profile_id = :me)`.
   - `private_summary` hanya bisa diakses lewat API agregat.
   - Role owner tidak menembus `private`, sesuai FR-6.
   - Update dan hapus mensyaratkan `owner_profile_id = :me`.
   - *Bukti:* test profil B tidak bisa list/get/update/delete catatan `private` milik A, dan untuk `private_summary` hanya mendapat agregat.
3. **Hapus fallback `vault.key`.**
   - `ensure_unlocked_key()` mengembalikan `VaultError::Locked` (`CAF-VAULT-001`).
   - Sediakan migrasi satu kali, dengan persetujuan pengguna, untuk DB yang terlanjur memakai `vault.key`, lalu hapus file itu.
   - Ubah test agar membuka vault dengan master password.
   - *Bukti:* test "terkunci → `CAF-VAULT-001`" dan test bahwa `vault.key` tidak pernah dibuat.
4. **Zeroize yang benar.**
   - Simpan kunci sebagai `Zeroizing<[u8; 32]>` atau `secrecy::SecretBox`.
   - Saat lock: zeroize in-place (`if let Some(k) = guard.as_mut() { k.zeroize() }`), baru set ke `None`.
   - Bangun string PRAGMA di `Zeroizing<String>`.
5. **Batas percobaan di core** untuk master password dan PIN, dengan back-off yang disimpan (PIN di dalam vault). Pakai `LockoutActive`. Hapus lockout versi localStorage.

### P1 — Fitur yang hanya UI atau rusak

6. **Satu sumber kebenaran untuk binding.**
   - Generate TypeScript dari Rust (`tauri-specta` atau `ts-rs`), atau samakan manual nama argumen (`old`/`new`, `password`, `data`, `id`) dan tipe (`AiChatResponse.message`, base64 vs `Vec<u8>`, `has_pin`, `avatar`).
   - Daftarkan atau hapus 25 command yang tidak ada. Untuk Pro: sembunyikan seluruh UI saat modul atau endpoint tidak di-set (FR-8).
   - Tambahkan arch test yang memindai `frontend/src` untuk `invoke('…')` dan gagal bila command tidak terdaftar.
7. **Backup v2.**
   - Header: magic, `format_version`, parameter KDF, salt acak.
   - Restore = `INSERT … ON CONFLICT DO UPDATE` dalam satu transaksi, termasuk tombstone, `pin_hash`, dan `change_log`. Gagal keras bila ada error.
   - Test round-trip (create → backup → wipe → restore → identik).
   - Backup terjadwal dengan clock yang bisa diinjeksi (S5.7b).
8. **Lock level 2.**
   - Alur: layar kunci → pemilih profil → PIN pad (target sentuh ≥ 44 px).
   - Role di `pin_required_for` wajib punya PIN.
   - Dua timer auto-lock.
   - Indeks profil di luar vault berisi nama + avatar saja, dengan opsi sembunyikan nama.
   - Mode `profiles = false` berperilaku seperti CATerm.
9. **Recovery code.**
   - DEK acak 32 byte, dibungkus oleh KEK(master) dan KEK(24 kata).
   - Ganti password = bungkus ulang DEK (tanpa rekey DB). Regenerasi mencabut kode lama.
   - Test: kode lama gagal setelah regenerasi, kode plaintext tidak ada di disk.

### P2 — Konvensi data dan generator

10. **Migration runner.** Tabel `schema_version`, migrasi bernomor (rentang framework vs produk), timestamp `INTEGER` unix ms, `change_log` mengikuti nama PRD.
11. **`change_log` otomatis.** Pilih salah satu: trigger `AFTER INSERT/UPDATE` per tabel sync-ready, atau satu helper `sync::write()` yang dipakai semua modul termasuk profiles.
12. **Test konvensi.**
    - Guard test yang membaca `sqlite_master` + `pragma_table_info` dan gagal bila ada tabel tanpa kolom wajib, dengan daftar pengecualian eksplisit (`schema_version`, `change_log`, `app_kv`).
    - Plain-header test.
    - Buat tabel `command_logs` atau hapus modul audit.
13. **xtask.**
    - Samakan skema dengan PRD FR-1 (atau perbarui PRD setelah keputusan pemilik, lihat pertanyaan di §6).
    - `#[serde(deny_unknown_fields)]` + validasi lengkap + unit test per kasus invalid.
    - `new-app`:
      - Salin berdasarkan `git ls-files` (allow-list), bukan WalkDir.
      - Tabel penggantian eksplisit: urut dari string terpanjang, pakai batas kata, ganti nama crate/package/storage key.
      - Implementasikan `--with-sample`.
      - Buat kunci minisign baru dengan crate `minisign`; private key ditulis di luar repo.
      - Placeholder route per menu, dry-run yang mencetak rencana.
      - Test E2E yang menjalankan STD di temp dir.

### P3 — Frontend dan kebersihan repo

14. **Shell dan design system.**
    - Komponen `TitleBar` di shell (disembunyikan di Android).
    - Drawer responsif di bawah `md`.
    - Varian `dark:` atau kelas berbasis token di semua halaman.
    - Map kelas literal 10 aksen yang dibaca dari `APP_CONFIG.accentColor`.
    - `PageHeader` dengan ikon + aksen. `PageContainer` di setiap halaman.
    - `ErrorState` + `BackendUnavailable`.
    - Settings registry.
15. **i18n.**
    - Pulihkan 134 key yang hilang, atau hapus komponen CATerm yang tidak dipakai.
    - Jadikan `t()` type-safe (`type Key = Paths<Dictionary>`).
    - Pindahkan semua string hard-coded ke kamus.
    - Helper format angka, mata uang, tanggal untuk `id` dan `en`.
16. **AI.**
    - Mode Off/BYO/Hosted. Tolak provider yang belum diimplementasikan. Jangan pernah mengirim key ke endpoint default provider lain.
    - Registry context provider + privacy level + dialog consent.
    - Scrub `prompt` juga.
    - Perluas scrubber: telepon Indonesia, NIK, NPWP, IBAN, kartu 13–19 digit dengan cek Luhn. Perbaiki kelas karakter regex email. Compile regex sekali (`OnceLock`) tanpa `unwrap`.
    - `CAF-AI-002` menggantikan respons palsu. Timeout HTTP.
    - `get_ai_settings` mengembalikan `has_api_key`, bukan key-nya.
    - Perbaiki `AiChatPanel` agar menampilkan `.message`.
17. **Purge sisa SSH/terminal.**
    - Hapus folder `permissions/autogenerated/` lalu biarkan `tauri-build` membuat ulang.
    - Hapus `nav.ts`, grup shortcut terminal/sftp, `terminalTheme`, `OsIcon`, `hostinger-logo.jpg`, prefs terminal, dokumen CATerm di root, file snap/flatpak/docker-compose CATerm, dan `build/bin/caterm` (`git rm`; pertimbangkan pembersihan history karena 15 MB).
    - Hapus alias `CatermError` dan dependency yatim (§1, butir 1.6).
18. **CI.** Tambahkan clippy `-D warnings`, `cargo deny`, guard (biner, > 1 MB, path absolut), E2E generator, pemeriksaan registry `invoke`, dan gitleaks.
19. **Proses.**
    - Salin `task.md` ke `Notes/task.md`.
    - Isi `Notes/evidence/S<n>.md` dengan output asli.
    - Centang hanya yang terverifikasi.
    - Pesan commit tidak boleh mengklaim "verified" tanpa file bukti.

---

## 4. Yang sudah sesuai (dengan bukti)

- Crate SSH/FTP/PTY dan `@xterm/*` hilang dari dependency (1.1–1.4).
- SQLCipher bundled, `PRAGMA key` mode raw, Argon2id m=64 MiB, t=3, p=4, canary AES-GCM (3.2).
- Data dir `~/.local/share/caframework` terpisah dari CATerm, dengan override `CAF_DATA_DIR`, dan test memakai data dir sementara (3.1).
- Hash PIN Argon2id (PHC, salt acak) di dalam vault dan tidak dikirim ke frontend (3.7).
- `notes`: UUIDv7, `rev` naik, soft delete, `change_log` dalam transaksi yang sama (4.2, 4.5, 4.6 untuk notes).
- Svelte 5 runes, store `*.svelte.ts`, Tailwind 4 dengan token sesuai kontrak (5.1, 5.2).
- Paritas kamus en/id dan `id: Dictionary` (5.3, sebagian).
- Envelope error `{ code, message, domain }`, golden test registry, arch test (core tanpa crate GUI, command tipis maks. 15 baris, kesesuaian `lib.rs`/`build.rs`/ACL).
- Crash: panic hook dipasang pertama, dump 0600 (unix), scrubber key/secret/IP/home/username, validasi id anti path traversal, "never ask again" di core.
- AI: key di DB terenkripsi (bukan localStorage), BYO OpenAI-compatible dan Ollama di core, redaksi email dan kartu 16 digit beserta test.

---

## 5. ✅ Checklist Perintah Verifikasi (dijalankan pemilik)

Supaya repo kerja tetap utuh, jalankan di clone, kecuali langkah generator yang memang harus membaca working tree asli (outputnya hanya ke `/tmp`).

```bash
git clone ~/Project/caframework /tmp/caf-audit && cd /tmp/caf-audit
```

### A. Set verifikasi standar (task.md)

```bash
cargo test --workspace                                    # harapan: 0 failed
cargo clippy --workspace --all-targets -- -D warnings     # prediksi: GAGAL di caf-core/src/ai.rs:99 dan :107 (clippy::unwrap_used)
cargo fmt --all -- --check
cargo deny check
cargo test -p caf-core --test arch
cargo test -p caf-core --test error_codes_unique
(cd frontend && npm ci && npm run check && npm run build)
cargo check -p caf-app
```

### B. Area 1 — purge

```bash
grep -E '^name = "(ssh2|libssh2-sys|russh|russh-keys|suppaftp|portable-pty|reqwest_dav|rusty-s3)"' Cargo.lock   # harapan: kosong
grep -c 'node_modules/@xterm' frontend/package-lock.json                                                    # harapan: 0
ls crates/caf-app/permissions/autogenerated | wc -l       # prediksi 113; seharusnya = jumlah command terdaftar (25)
git grep -n -I -i -E 'ssh|sftp|\bscp\b|\bftp\b|webdav|xterm|terminal|tunnel' -- crates frontend/src snap flatpak docker-compose.test.yml ':!crates/caf-app/permissions'
git grep -c -I 'CatermError' -- crates | head
```

### C. Area 2 — app.toml dan xtask

```bash
# 1) Codegen reproducible (di clone)
cargo run -p caf-xtask -- codegen && git diff --exit-code frontend/src/lib/generated && echo REPRODUCIBLE

# 2) Contoh app.toml PRD FR-1: salin blok toml dari prd.md §FR-1 ke /tmp/prd-app.toml, lalu
cargo run -p caf-xtask -- codegen --config /tmp/prd-app.toml
#    prediksi: error "missing field `bundle_id`" (skema tidak sesuai PRD)

# 3) Key tak dikenal harus ditolak
cp app.toml /tmp/unknown.toml && printf '\n[foo]\nbar = 1\n' >> /tmp/unknown.toml
cargo run -p caf-xtask -- codegen --config /tmp/unknown.toml
#    prediksi: SUKSES (seharusnya gagal). Catatan: codegen menulis ke <folder config>/frontend/src/lib/generated,
#    jadi file hasilnya muncul di /tmp/frontend/... Hapus setelahnya: rm -rf /tmp/frontend

# 4) new-app dari working tree asli (repo hanya dibaca; output ke /tmp; build ke target/)
cd ~/Project/caframework
cargo run -p caf-xtask -- new-app --config app.toml --out /tmp/caf-demo --dry-run
cargo run -p caf-xtask -- new-app --config app.toml --out /tmp/caf-demo --with-sample
du -sh /tmp/caf-demo                                     # prediksi: ±1,7 GB (dist/, crates/caf-app/gen/, build/)
ls -a /tmp/caf-demo | grep -E '^\.git'                   # prediksi: kosong (.github dan .gitignore hilang)
grep -rIl -i -E 'caterm' /tmp/caf-demo --exclude-dir={target,node_modules,gen,dist} | head
grep -n '"identifier"\|"pubkey"' /tmp/caf-demo/crates/caf-app/tauri.conf.json   # pubkey sama dengan framework
cd /tmp/caf-demo && cargo check -p caf-app && (cd frontend && npm ci && npm run check && npm run build)
```

### D. Area 3 — keamanan (aplikasi terpasang di laptop)

```bash
ls -la ~/.local/share/caframework/
test -f ~/.local/share/caframework/vault.key && echo "TEMUAN: vault.key (kunci plaintext) ada"
head -c 16 ~/.local/share/caframework/caframework.db | xxd   # TIDAK boleh berbunyi "SQLite format 3."
stat -c '%a %n' ~/.local/share/caframework/*               # periksa permission file vault
```

Uji bypass capability di DevTools build debug (`cargo tauri dev`), setelah unlock:

```js
// Prediksi saat ini: SUKSES. Harapan setelah perbaikan: error CAF-AUTH-*
await window.__TAURI_INTERNALS__.invoke('list_notes', { callerProfileId: 'profil-lain', isOwner: true });
await window.__TAURI_INTERNALS__.invoke('save_profile', { input: { name: 'X', role: 'owner', avatar: null, pin: null }, callerProfileId: 'siapa-saja' });
// Prediksi: error "missing required key old" (mismatch parameter)
await window.__TAURI_INTERNALS__.invoke('change_master_password', { currentPassword: 'a', newPassword: 'b' });
```

### E. Area 4 — sync-ready

```bash
git grep -n -i 'CREATE TRIGGER' -- crates            # prediksi: kosong
git grep -n 'change_log' -- crates/caf-core/src/profiles.rs   # prediksi: kosong
git grep -n 'command_logs' -- crates/caf-core/src/db.rs       # prediksi: kosong (tabel audit tidak dibuat)
cargo test -p caf-core notes::                        # lulus, tetapi tidak memeriksa isi change_log atau rollback
```

### F. Area 5 — frontend

```bash
cd frontend
grep -o 'commands::[a-z_]*' ../crates/caf-app/src/lib.rs | sed 's/commands:://' | sort -u > /tmp/reg.txt
grep -rhoE "invoke(<[^>]*>)?\(\s*['\"][a-z_]+['\"]" src | sed -E "s/.*\(\s*['\"]([a-z_]+)['\"].*/\1/" | sort -u > /tmp/inv.txt
comm -13 /tmp/reg.txt /tmp/inv.txt                     # prediksi: 25 command tidak terdaftar
grep -c 'dark:' src/routes/+layout.svelte src/routes/+page.svelte src/routes/settings/+page.svelte   # prediksi: 0
```

Cek visual: jalankan app, lalu periksa apakah layar kunci menampilkan key mentah `lock.*`, apakah ada tombol tutup/minimize setelah unlock, dan apakah shell berubah saat tema terang dipilih.

### G. Area 6 — AI

```bash
cargo test -p caf-core ai::
ollama serve & ollama pull llama3.2
```

Lalu di Settings → AI pilih provider `ollama`, endpoint `http://localhost:11434`, dan kirim pesan. Prediksi: tampil `[object Object]`.

### H. Release binary, launcher, tag, kebersihan

```bash
cd ~/Project/caframework
ls -la target/release/caf-app && file target/release/caf-app
./target/release/caf-app 2>&1 | grep CAFRAMEWORK_COLD_START_MS        # ukur cold start (NFR <= 2 dtk)
ls ~/.local/share/applications /usr/share/applications 2>/dev/null | grep -i -E 'caf|caframework'
desktop-file-validate ~/.local/share/applications/<file>.desktop      # jika ditemukan
git tag -l 'v0.1.0'                                                   # prediksi: kosong
git ls-files -z | xargs -0 du -b | awk '$1 > 1048576'                 # prediksi: build/bin/caterm
gitleaks detect --source . --log-opts="--all" -v
ls Notes/evidence/ Notes/task.md                                      # prediksi: kosong / tidak ada
```

---

## 6. Batasan audit dan pertanyaan untuk pemilik

**Batasan.** Audit ini statis. Tidak ada build, test, atau aplikasi yang dijalankan, jadi butir *prediksi* wajib dikonfirmasi dengan §5. Repo CATerm tidak terhubung, sehingga kesamaan visual dengan CATerm (S4.1) tidak dibandingkan. Android (FR-13), aksesibilitas, dan performa di luar 7 area.

**Pertanyaan untuk pemilik.** Masing-masing disertai rekomendasi default.

1. Skema `app.toml` mana yang menjadi acuan: PRD FR-1, atau skema implementasi yang punya `[window]`/`[security]`? *Rekomendasi:* PRD FR-1 sebagai dasar, ditambah `[window]` sebagai bagian opsional. Generator wajib bisa membaca `app.toml` CACash yang ditulis sesuai PRD.
2. Apakah role `owner` boleh membaca catatan `private` milik profil lain? PRD menyatakan `private` = hanya pemilik catatan, sedangkan kode memberi akses ke owner. *Rekomendasi:* ikuti PRD; owner hanya mendapat agregat untuk `private_summary`.
3. Commit `7ec9e64` menyebut "desktop launcher". Di mana file `.desktop` yang dimaksud? *Rekomendasi:* jadikan bagian repo (`packaging/linux/<slug>.desktop` hasil generate dari `app.toml`) agar ikut dalam audit dan generator.
