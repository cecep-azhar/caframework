# TASK REMEDIASI CAFRAMEWORK (ROADMAP PERBAIKAN v0.1.0)
**Berdasarkan Hasil Audit Statis: 2 Oktober 2026**
**Status:** DRAF RENCANA KERJA (Belum Dieksekusi / Read-Only Planning)

---

## 🎯 Ringkasan Status Audit
- **Total Butir Dinilai:** 73 Butir
- **Hasil:** 16 PASS · 16 PARTIAL · 40 FAIL · 1 BELUM DIVERIFIKASI
- **Temuan Kritis/Tinggi:** 1 Temuan Kritis (C-1) & 10 Temuan Tinggi (T-1 s.d T-10)
- **Status Rilis:** ⛔ Belum layak dirilis sebagai v0.1.0 (Definition of Done PRD §9 belum terpenuhi).

---

## 📋 DAFTAR FASE PENGERJAAN & CHECKLIST (P0 – P3)

---

### 🚨 FASE 0: BLOCKER KEAMANAN & CORE AUTH (P0) — *Kritis & Mutlak Sebelum Fitur Lain*

- [ ] **P0.1: Arsitektur Session & Capability Check di Core (C-1)**
  - [ ] Buat `SessionState` terkelola di Rust core (`active_profile_id`, `active_role`, `unlocked_at`).
  - [ ] State hanya dapat diisi melalui pemanggilan `unlock_vault` + `verify_pin` yang valid.
  - [ ] Hapus parameter rentan `caller_profile_id` dan `is_owner` dari semua signature Tauri command frontend.
  - [ ] Pasang guard role di Rust core: Operasi owner-only (`save_profile` profil lain, `reset_vault`, `change_master_password`, export/import backup) wajib menolak pemanggil non-owner dengan `CAF-AUTH-001` / `CAF-AUTH-003`.
  - [ ] Buat unit/integration test: Panggilan command sensitif oleh profil non-owner wajib gagal dengan error code auth.

- [ ] **P0.2: Helper Filter Visibilitas & Data Isolation di Core (C-1)**
  - [ ] Implementasikan query SQL standar di core: `deleted_at IS NULL AND (visibility = 'shared' OR owner_profile_id = :me)`.
  - [ ] Terapkan aturan ketat: Role `owner` dilarang membaca konten baris `private` milik profil lain (hanya dapat agregat `private_summary`).
  - [ ] Operasi update dan delete wajib memvalidasi `owner_profile_id == active_session.profile_id`.
  - [ ] Buat unit test: Profil B dilarang membaca/mengubah catatan private profil A.

- [ ] **P0.3: Eliminasi Kunci Plaintext `vault.key` & Perbaikan Status Locked (T-1)**
  - [ ] Hapus logika fallback `load_or_create_local_key` dan pembuatan file `vault.key` di `vault.rs`.
  - [ ] Ubah `ensure_unlocked_key()` agar mengembalikan `VaultError::Locked` (`CAF-VAULT-001`) jika vault belum dibuka.
  - [ ] Sediakan mekanisme migrasi satu kali (dengan konfirmasi) bagi DB existing, lalu hapus `vault.key`.
  - [ ] Refactor seluruh unit test DB agar membuka vault menggunakan master password.
  - [ ] Buat test guard: Memastikan status vault terkunci mengembalikan `CAF-VAULT-001` dan `vault.key` tidak pernah dibuat di disk.

- [ ] **P0.4: Implementasi Zeroization Memori yang Benar (S-1)**
  - [ ] Bungkus kunci master & DB key menggunakan `Zeroizing<[u8; 32]>` / `secrecy::SecretBox`.
  - [ ] Pastikan saat aksi lock dipanggil, memori kunci di-zeroize in-place (`k.zeroize()`) sebelum referensi di-`None`-kan.
  - [ ] Pastikan string PRAGMA key dan parameter password di-zeroize setelah eksekusi.

- [ ] **P0.5: Rate Limiting & Back-off Percobaan Sandi/PIN di Core (T-2, S6.3)**
  - [ ] Pindahkan counter percobaan gagal & lockout timer dari localStorage frontend ke state terenkripsi di core.
  - [ ] Return error `CAF-VAULT-002` (`VaultError::LockoutActive`) jika batas percobaan terlampaui.
  - [ ] Hapus sisa kode lockout yang tersimpan di localStorage browser.

---

### 🛡️ FASE 1: PERBAIKAN WIRING FRONTEND-BACKEND & LOGIKA RUSAK (P1)

- [ ] **P1.1: Sinkronisasi Kontrak Command & Binding IPC (T-4)**
  - [ ] Hapus/rapikan 25 nama command yang dipanggil frontend tetapi tidak terdaftar di Rust (atau daftarkan handler jika fiturnya wajib).
  - [ ] Sinkronkan nama parameter Tauri (camelCase di JS vs snake_case di Rust):
    - `change_master_password`: `{ old, new }` vs `{ currentPassword, newPassword }`
    - `export_encrypted_backup`: `{ password }` vs `{ passphrase }` & handling return payload
    - `import_encrypted_backup`: `{ data, password }` vs `{ encryptedB64, passphrase }`
    - `dismiss_crash_report`: `{ id, neverAgain }` vs `{ reportId, neverAgain }`
  - [ ] Sinkronkan tipe data return:
    - `ai_chat`: Parse objek `{ message, redactions_applied, tokens_used }` di TS, bukan menganggap `string` mentah.
    - Sinkronkan struct `AiSettings` (hilangkan field fiktif `guardrails_enabled` di TS, gunakan `temperature` & `privacy_redaction_enabled`).
    - Sinkronkan model Profil: `avatar` (bukan `avatar_url`) dan ganti pembacaan `pin_hash` menjadi `has_pin`.
  - [ ] Buat arch test: Scanner otomatis yang memverifikasi setiap `invoke('...')` di frontend terdaftar di `generate_handler!` Rust.

- [ ] **P1.2: Pembenahan Backup & Restore v2 (T-5, FR-9)**
  - [ ] Buat format header backup: Magic bytes, `format_version`, salt acak (bukan hardcoded), dan metadata KDF.
  - [ ] Perbaiki logika Restore: Gunakan `INSERT ... ON CONFLICT DO UPDATE` dalam 1 transaksi utuh (termasuk memulihkan data ke vault kosong, menyertakan `change_log` dan tombstone).
  - [ ] Buat test round-trip: Buat data -> Export backup -> Wipe vault -> Import restore -> Verifikasi data 100% identik.
  - [ ] Implementasikan backup berkala (scheduled backup) dengan injectable clock.

- [ ] **P1.3: Implementasi Lock Level 2 & Profil PIN Pad (T-2, FR-6)**
  - [ ] Bangun alur UI 2 tahap: Layar Buka Vault (Master Password) -> Pemilih Profil -> Input PIN (Touch target >= 44px).
  - [ ] Enforce validasi PIN untuk role yang diwajibkan di konfigurasi.
  - [ ] Implementasikan 2 timer auto-lock independen (Inactivity Lock & Hard Vault Lock).
  - [ ] Simpan metadata profil publik (nama & avatar) terpisah untuk pemilih profil dengan opsi sembunyikan nama.

- [ ] **P1.4: Implementasi Recovery Code 24 Kata & DEK Wrapping (T-3, FR-4)**
  - [ ] Generate Data Encryption Key (DEK) 32-byte acak untuk SQLCipher.
  - [ ] DEK dibungkus (wrapped) dua kali: oleh KEK Master Password dan KEK Recovery Code (24 kata BIP-39).
  - [ ] Logika ganti master password: Cukup re-wrap DEK tanpa perlu `PRAGMA rekey` ke seluruh database.
  - [ ] Buat test: Recovery code lama otomatis hangus saat regenerasi, dan recovery code tidak pernah disimpan plaintext di disk.

---

### 📦 FASE 2: DATA SYNC CONVENTION & CODEGEN XTASK (P2)

- [ ] **P2.1: Migration Runner & Skema Sync Standar (T-7, S-3, FR-5)**
  - [ ] Bangun migration runner yang membaca dan menulis ke tabel `schema_version`.
  - [ ] Standarisasi timestamp: Gunakan tipe `INTEGER` (unix millisecond UTC), bukan campuran string RFC3339 / SQLite `datetime('now')`.
  - [ ] Standarisasi penamaan tabel dan kolom `change_log`: `entity`, `op`, `rev`, `at` sesuai spesifikasi PRD FR-5.
  - [ ] Pastikan operasi profil (`save_profile`, `delete_profile`) selalu menulis baris ke `change_log` dan menaikkan nilai `rev`.

- [ ] **P2.2: Automasi Trigger / Helper Change Log (P2-11)**
  - [ ] Buat helper terpusat `sync::write()` atau trigger SQLite otomatis `AFTER INSERT/UPDATE/DELETE` untuk seluruh tabel sync-ready.
  - [ ] Selesaikan pembuatan tabel audit `command_logs` (hapus kolom warisan `host_id`) atau rapikan modul audit.

- [ ] **P2.3: Guard Tests Konvensi Basis Data (T-7, S5.4, S5.6)**
  - [ ] Buat guard test yang menginspeksi `sqlite_master` & `pragma_table_info` untuk memverifikasi kolom wajib (`id`, `rev`, `created_at`, `updated_at`, `deleted_at`, `origin_device_id`, `owner_profile_id`, `visibility`) pada semua tabel data.
  - [ ] Buat test `plain-header`: Memastikan file `.db` tidak memiliki header `"SQLite format 3"` (terbukti terenkripsi penuh).

- [ ] **P2.4: Pembenahan Generator `new-app` & Skema `app.toml` (T-6, FR-1, FR-12)**
  - [ ] Harmonisasikan parser `app.toml` dengan spesifikasi PRD FR-1 (`[app]`, `[modules]`, `[profiles]`, `[endpoints]`, `[[menus]]`) dan pasang `#[serde(deny_unknown_fields)]`.
  - [ ] Perbaiki engine rename pada `caf-xtask`:
    - Ganti strategi salin dari `WalkDir` ke allow-list (`git ls-files`).
    - Buat tabel penggantian terurut (string terpanjang dulu, word boundary, rename crate `caf-*`, package `caf-frontend`, bundle_id, dan storage key).
    - Exclude direktori berat/sampah: `dist/`, `gen/`, `build/bin/`, artefak CATerm.
    - Implementasikan flag `--with-sample` secara nyata.
    - Generate pasangan kunci `minisign` baru untuk updater per aplikasi yang dibuat.
  - [ ] Buat unit test skema `app.toml` dan test E2E generator di direktori temporer.

---

### 🎨 FASE 3: UI/UX, SHELL, I18N, AI & KEBERSIHAN REPO (P3)

- [ ] **P3.1: Perbaikan Shell Window & UI Design System (T-8, FR-2)**
  - [ ] Buat komponen custom `TitleBar` di shell aplikasi lengkap dengan drag region dan kontrol window (minimize, maximize, close) untuk desktop frameless.
  - [ ] Tambahkan drawer navigasi responsif untuk layar di bawah breakpoint `md`.
  - [ ] Perbaiki theme provider: Pasang varian `dark:` atau token dinamis di seluruh shell, Notes, Settings, dan state components.
  - [ ] Terapkan aksen dinamis dari `APP_CONFIG.accentColor` dengan map 10 kelas literal.
  - [ ] Gunakan `PageContainer` secara konsisten di semua halaman.
  - [ ] Sediakan komponen `ErrorState` dan `BackendUnavailable` yang fungsional.
  - [ ] Implementasikan Settings Registry dinamis sesuai `app.toml`.

- [ ] **P3.2: Perbaikan Kamus Bahasa & i18n (T-8, 5.3, FR-10)**
  - [ ] Lengkapi 134 translation keys yang hilang di `en.ts` dan `id.ts` (terutama grup `lock.*`, `pro.*`, `feedback.*`, `crash.*`, `about.*`).
  - [ ] Jadikan fungsi helper `t()` type-safe (`Paths<Dictionary>`).
  - [ ] Pindahkan seluruh teks hardcoded di komponen UI ke kamus bahasa.
  - [ ] Buat helper format angka, mata uang, dan tanggal untuk locale `id` dan `en`.

- [ ] **P3.3: Perbaikan Privasi & Fungsionalitas AI Assistant (T-10, FR-7)**
  - [ ] Implementasikan mode AI: `Off`, `BYO`, dan `Hosted`.
  - [ ] Isolasi endpoint: Cegah pengiriman API key provider lain ke endpoint default OpenAI.
  - [ ] Tambahkan sanitasi PII pada `prompt` (bukan hanya pada `context`).
  - [ ] Tingkatkan regex PII scrubber: Dukungan nomor HP Indonesia, NIK, NPWP, kartu 13–19 digit (Luhn check), dan perbaiki regex email.
  - [ ] Hilangkan `.unwrap()` pada kompilasi regex (`OnceLock`) untuk mematuhi `#![deny(clippy::unwrap_used)]`.
  - [ ] Hilangkan respons sukses palsu saat API key kosong -> kembalikan error `CAF-AI-002`.
  - [ ] Sembunyikan API key saat dibaca di frontend (hanya kirim status boolean `has_api_key`).
  - [ ] Perbaiki `AiChatPanel.svelte` agar merender properti teks `.message` dari respons AI.

- [ ] **P3.4: Pembersihan Sisa File & Dependency Basi CATerm / SSH (S-4, S-5, 1.5)**
  - [ ] Hapus 88 file permission basi di `crates/caf-app/permissions/autogenerated/`.
  - [ ] Hapus file & komponen basi: `lib/nav.ts`, shortcuts grup terminal/sftp, `terminalTheme`, `OsIcon.svelte`, `static/hostinger-logo.jpg`, prefs terminal di Rust.
  - [ ] Hapus binary dan dokumen ter-track: `build/bin/caterm`, file snapcraft/flatpak caterm, dan dokumen caterm lama di root repo.
  - [ ] Hapus alias `CatermError` di seluruh codebase Rust dan bersihkan dependency yatim.
  - [ ] Bersihkan path absolut lokal di file dokumentasi Android.

- [ ] **P3.5: Penguatan CI Workflow & Audit Tooling (S-5, 7.2)**
  - [ ] Tambahkan step wajib di `.github/workflows/ci.yml`:
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo deny check`
    - Guard pengecekan file binary > 1MB dan anti-hardcoded local absolute paths.
    - E2E testing generator `new-app`.
    - Script verifikasi pemanggilan `invoke` vs handler Rust.

- [ ] **P3.6: Standarisasi Bukti Pengerjaan (Evidence) & Version Tagging (T-9, S15)**
  - [ ] Buat folder `Notes/evidence/` untuk menyimpan log eksekusi nyata dari tiap fase pengerjaan.
  - [ ] Jangan menandai checklist selesai tanpa rekaman bukti output asli.
  - [ ] Buat git tag resmi `v0.1.0` setelah seluruh Definition of Done (DoD) PRD §9 terpenuhi dan terverifikasi.

---

## 🔒 Konfirmasi & Status Saat Ini
- **Status Eksekusi Kode:** 🛑 **DIHENTIKAN / BELUM DIJALANKAN** sesuai arahan Prof. Cecep.
- Berkas `TASK.md` siap direview dan dijadikan acuan kerja ketika Prof. Cecep memberikan instruksi mulai.
