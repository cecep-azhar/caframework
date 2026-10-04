# CAFramework — Remediation Task Plan v2.1 (`TASK_FIX.md`)

> **Status:** ready to execute. Owner decisions in §3 confirmed on 4 October 2026 (v2.1 adds the PRD v1.1 foundations: brand, RBAC, super role, mandatory backup, quality bars).
> **Date:** 4 October 2026 · **Owner:** Cecep Azhar
> **Inputs:** `prd.md` v1.1 · `task.md` v1.0 (stages S0–S15) · `prompt-dev.md` v1.0 · `audit/2026-10-02-audit-statis-caframework.md` (repo HEAD `7ec9e64`)
> **Next plan:** `TASK_FEATURES.md` (v0.2.0) starts after F20.
> **Run with:** `prompt-fix.md`, which supplements `prompt-dev.md` and does not replace it. This file says *what* must change and *how each change is proven*; `prompt-fix.md` says *how the agent operates*.
> **Replaces:** the earlier remediation draft (P0–P3 checklist, also saved as `TASK.md` / `task-remediasi-audit.md`). Appendix A reviews its coverage.
> **Language rule (PRD):** instructions in English; every report to the owner in Bahasa Indonesia.

## Ringkasan untuk pemilik

**Apakah draf lama sudah merepresentasikan perbaikan? Belum.** Arahnya benar karena mengikuti rekomendasi P0–P3 di audit, tetapi belum bisa dipakai agent untuk bekerja sendiri.

- **Cakupan kurang.** Dari 57 butir audit yang berstatus FAIL, PARTIAL, atau BELUM DIVERIFIKASI, draf lama mencakup penuh 35, sebagian 10, dan **tidak mencakup 12**. Dari 25 temuan (C/T/S/R), 10 tercakup penuh, 7 sebagian, dan **8 tidak ada sama sekali**: S-6 (endpoint privat dan `app_id`), S-7 (rahasia di argumen CLI), S-8 (scrubber crash), S-9 (Android), R-1, R-2, R-4, dan R-5. Saklar modul `app.toml`, privacy level dan guardrail AI, Android (FR-13), dan launcher desktop tidak disentuh.
- **Tidak ada bukti per butir.** Isinya hanya checkbox, tanpa kriteria terima, perintah verifikasi, atau file bukti. Pola inilah yang membuat agent bisa melaporkan "selesai" padahal baru UI-nya.
- **Urutannya bermasalah.** Folder bukti baru dibuat di fase terakhir. Perbaikan clippy ditaruh di P3, padahal clippy sudah gagal sejak awal, sehingga tidak ada gate yang bisa hijau. Capability check (P0) membutuhkan daftar role dari skema `app.toml` yang baru dikerjakan di P2. Migration runner dibuat setelah pekerjaan yang mengubah skema dan kunci.
- **Ada desain yang tidak bisa berjalan.** Penghitung gagal master password disimpan "di state terenkripsi", padahal vault masih terkunci saat password salah. Backup terjadwal berbasis passphrase tidak bisa berjalan tanpa pengguna. Kode `CAF-VAULT-002` tidak bersumber dari registry maupun audit.

**Rencana baru.** 21 tahap (F0–F20). Setiap task punya langkah kerja, kriteria terima, perintah verifikasi, file bukti, dan ID `task.md` asalnya. Ke-73 butir audit dan 25 temuan dipetakan di §5, dan setiap FR/NFR PRD di §6. F0 hanya mengukur kondisi nyata dan mengonfirmasi semua *prediksi* audit, tanpa mengubah kode.

**Pembaruan v2.1 (4 Oktober 2026).** Fitur baru (manajemen role saat runtime, integrasi API/GCC, scheduler, import/export entitas, paket Pro, wizard) dikerjakan di `TASK_FEATURES.md` untuk v0.2.0. Rencana ini hanya memasang fondasinya agar tidak dibongkar dua kali: skema `app.toml` v1.1 lengkap (termasuk `[brand]` wajib dan RBAC), pemeriksaan izin RBAC di core, aturan super admin boleh membaca semua data (dengan audit log), backup yang tidak bisa dimatikan, dan syarat kualitas stabil, mudah dirawat, dan scalable (§1.10).

**Keputusan Anda (4 Oktober 2026)** tercatat di §3. Yang berbeda dari rekomendasi awal hanya dua: vault lama **tidak dimigrasi** (D-5), dan agent berjalan dalam **satu sesi panjang** (D-8). Untuk D-8, `prompt-fix.md` dibuat bisa dilanjutkan dari titik mana pun lewat `Notes/fix-ledger.md`.

**Yang perlu Anda lakukan:**
1. Isi blok konfigurasi di `prompt-fix.md` §0, lalu kirim pesan mulai (Appendix A di prompt).
2. Kerjakan antrian OWNER-VERIFY di ledger saat laptop menyala: aplikasi berjalan, Android, cold start, perbandingan visual dengan CATerm, dan menghapus folder lokal `dist/` serta `crates/caf-app/gen/android`.
3. Sebelum F20, kirim `app.toml` CACash. Kalau belum ada, agent memakai contoh PRD FR-1 sebagai pengganti dan butir CACash di DoD tetap terbuka.

**Tetap di tangan Anda (Tier 2):** merge, tag `v0.1.0`, rewrite history, memindah `Notes/` ke repo privat, menghapus file di luar repo atau yang tidak ter-track, kunci rilis, dan publikasi.

---

## 0. How to read and run this file

- 21 stages, `F0`…`F20`, executed in order. Execution mode is D-8: one long session that continues through stages automatically and resumes from `Notes/fix-ledger.md` after any interruption (`prompt-fix.md` §4).
- Each stage header lists: **Fixes** (audit IDs, PRD refs, original `task.md` S-IDs), **Runner** (§1.7), **Depends on**, **Size** (S/M/L).
- Each task `F<stage>.<n>` has **Do** (steps), **Accept** (criteria that must be proven) and **Verify** (commands whose raw output becomes evidence). Each stage ends with a **Gate**.
- Status lives in `Notes/fix-ledger.md`. Tick a box in this file only together with a link to the evidence that proves it.
- `~71` means "around line 71 at commit `7ec9e64`". Re-locate by symbol before editing (§1.6).
- In shell snippets, `EVID` is `$REPO_DIR/Notes/evidence/fix`.
- `CORE` = `crates/caf-core/src`, `APP` = `crates/caf-app/src`, `XT` = `crates/caf-xtask/src`, `FE` = `frontend/src`.

## 1. Global rules

### 1.1 Verification sets (VS)

These extend the "STD" set in `task.md`; STD remains valid and is included here.

| Set | Commands | Runner |
|---|---|---|
| VS-R (Rust) | `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets --exclude caf-app -- -D warnings` · `cargo test --workspace --exclude caf-app` (includes `caf-core --test arch` and `--test error_codes_unique`) · `cargo deny check` | any |
| VS-F (frontend) | `cd frontend && npm ci && npm run check && npm run build && npm run test` (`test` exists from F1.6) | any |
| VS-G (guards) | `cargo run -p caf-xtask -- guard` (from F1.3) · `cargo run -p caf-xtask -- codegen --check` (from F3.4) | any |
| VS-A (app crate) | `cargo check -p caf-app` · `cargo clippy -p caf-app --all-targets -- -D warnings` · `cargo test -p caf-app` | x1-bench, or CI job `app` |
| VS-E (generator E2E) | `cargo test -p caf-xtask --test new_app_e2e -- --ignored --nocapture` (from F18) | any, ≥ 6 GB free disk |

"VS green" means every command exits 0. A stage gate requires VS-R, VS-F and VS-G green on the runner, plus VS-A green on x1-bench or in the CI `app` job for the stage's last commit. If VS-A cannot run, the stage is `GATE_PASSED (agent)` and VS-A goes to the owner queue (§1.5).

### 1.2 When a task counts as done

1. Code and tests are committed on the work branch.
2. Every **Accept** box is proven by raw output in the evidence file (command plus output), not by a sentence.
3. VS is green and no ratchet got worse (§1.3).
4. A user-facing feature shows the full wiring chain, otherwise it is **UI-ONLY** and the task fails:
   core function → core test → registered command with a policy class (F7.3) → generated binding in `FE/lib/generated/commands.ts` → wrapper in `FE/lib/api/` → UI call site → IPC contract test (F7.6) → owner smoke test with output, or an automated UI test (F14.9) where the stage allows it.
5. No test, ratchet entry, lint, `deny.toml` rule or CI step was weakened, deleted, skipped or `#[ignore]`d to make something pass. The only allowed `#[ignore]` is the slow E2E in VS-E.
6. A negative test exists for every guard: a guard counts only after evidence shows it failing on a deliberately bad fixture.

### 1.3 Ratchets (known debt that may only shrink)

Files live in `guards/ratchet/`, one violation per line, each with `# resolves-in: F<n>`. A ratchet check fails when (a) a violation is not listed, or (b) a listed entry no longer violates (stale entries must be deleted in the same commit).

| File | What it tracks | Audit baseline | Target |
|---|---|---|---|
| `unregistered_invokes.txt` | command names invoked from `FE` with no handler in `generate_handler!` | 25 (T-4) | 0 at F12 |
| `invoke_outside_api.txt` | `invoke(` outside `FE/lib/api/**` and `FE/lib/generated/**` | measured in F0 | 0 at F7 |
| `missing_i18n_keys.txt` | literal `t('<key>')` not present in `en.ts` | 134 (5.3) | 0 at F15 |
| `hardcoded_strings.txt` | user-visible text in `.svelte` markup not passed through `t()` | measured in F1 | 0 at F15 |
| `raw_color_classes.txt` | raw palette classes (`indigo-*`, hard-coded neutral/dark shades) outside `app.css`, `accent.ts` and state colours | 62 `indigo-*` (5.5) + measured | 0 at F14 |
| `raw_sync_writes.txt` | SQL `INSERT INTO`/`UPDATE`/`DELETE FROM` on a sync table outside `CORE/sync.rs` and migrations | measured in F1 | 0 at F5 |
| `oversized_or_binary.txt` | tracked binaries/DBs and files > 1 MB | `build/bin/caterm` (S-5) + measured | 0 at F2 |
| `absolute_paths.txt` | dev-machine absolute paths (`/home/`, `/Users/`, `C:\`, `~/Project/`) outside `Notes/` | 4+ (S-5) | 0 at F2 |
| `forbidden_tokens_allowlist.txt` | `caterm\|ssh\|sftp\|scp\|ftp\|webdav\|xterm\|terminal\|tunnel` outside `Notes/`, lockfiles and `guards/` | measured in F0 | permanent entries only, each with a reason |

### 1.4 Evidence

- One file per stage: `Notes/evidence/fix/F<n>.md` (template in `prompt-fix.md` §6.2). Raw logs: `Notes/evidence/fix/logs/F<n>-<slug>.log`.
- Paste outputs verbatim. If a log exceeds 900 KB, keep its first and last 300 lines plus every line matching `error|warning|FAILED|panicked|failed`, and record the full log's byte size and `sha256sum`.
- Any statement without linked evidence is reported as **BELUM DIVERIFIKASI**.
- Commit messages never say "verified", "tests pass" or "done" unless the commit also adds or updates the evidence file that proves it (T-9).

### 1.5 Owner-verify protocol

Some checks need the owner's laptop or a person: running the GUI, Android, cold-start timing, visual comparison with CATerm. The agent:

1. adds an `OWNER-VERIFY` entry to the ledger (template in `prompt-fix.md` §6.4) with copy-paste commands, the expected result, and where to paste the output;
2. starts every smoke command with a throwaway data dir, `export CAF_DATA_DIR="$(mktemp -d)"`, and never uses the owner's real `~/.local/share/caframework`;
3. marks the stage `GATE_PASSED (agent)`. It becomes `DONE` only when the owner's output is recorded. Later stages may proceed (D-8) unless the entry is marked `BLOCKING`.

### 1.6 Anchors and drift

Audit line numbers refer to `7ec9e64`. Before editing, locate code by symbol (`git grep -n '<symbol>'`) and record moved or changed anchors in the stage evidence. If the code a finding describes no longer exists, prove it with grep output instead of assuming it was fixed.

### 1.7 Runner labels

- `any`: a headless Linux runner with stable Rust (toolchain from `rust-toolchain.toml`) and Node LTS.
- `x1-bench`: the owner's ThinkPad X1 Yoga Gen 3, with Tauri desktop dependencies, a display, the Android SDK and the performance baseline. Stages that need it record VS-A and GUI checks as OWNER-VERIFY when the agent is not running there.

### 1.8 Errors and codes

- Add new variants to the existing domain enums. The golden test (`error_codes_unique`) assigns and freezes numbers; regenerate the registry and `docs/error-codes.md` in the same commit.
- This plan names variants, not numbers. The only codes it cites are those the audit cites (`CAF-VAULT-001` for `VaultError::Locked`, `CAF-AUTH-001`/`003`, `CAF-AI-002`), and they must be confirmed against the registry before use. `CAF-VAULT-002` from the old draft has no source and must not be assumed.
- No code path may return a `Generic` (`*-000`) variant for an expected failure (R-5). Expected failures include wrong password, wrong PIN, lockout, locked vault, missing session, permission denied, not found, validation error, module disabled, network, timeout, quota, unsupported version and integrity failure.
- Frontend reads errors only through `errorText(err)`; never `String(err)` (`prompt-dev.md` §7).

### 1.9 Dependencies

Every new crate or npm package is recorded in the stage report with version, licence and reason, and must pass `cargo deny check` / `npm audit --omit=dev --audit-level=high`. Prefer crates already in `Cargo.lock`. Remove a dependency in the same commit as its last use.

### 1.10 Quality bars: stable, maintainable, scalable (PRD §6)

These apply to every task from the stage that introduces them onwards. A violation fails the gate like a failing test.

| Bar | Enforced by | From |
|---|---|---|
| No `unwrap`/`expect`/`panic!`/`unreachable!`/unchecked indexing in non-test core code | clippy `unwrap_used`, `expect_used`, `panic`, `unreachable`, `indexing_slicing` denied in `caf-core` and `caf-xtask` (`#![cfg_attr(not(test), deny(...))]`) | F1.7 |
| Coverage never decreases; targets core ≥ 80 %, frontend ≥ 70 % by F20 | `cargo llvm-cov --workspace --exclude caf-app --fail-under-lines <ratchet>`; `vitest --coverage` thresholds; ratchet file `guards/ratchet/coverage.txt` | F1.7 |
| Source files ≤ 600 lines, functions ≤ 80 lines | `guard --only size` with listed exceptions (`file_size_allowlist.txt`, shrink-only) | F1.7 |
| Parsers of external input have property/fuzz tests | `proptest` suites (≥ 10 000 cases in CI) for `app.toml`, backup files, keyring JSON, HTTP/licence payloads, import files | F3, F6, F11, (v0.2: G2, G4, G5) |
| Atomic file writes | one helper `paths::write_private_atomic()`; guard forbids `std::fs::write` in `caf-core` outside it | F4.7 |
| DB integrity checked on unlock | `PRAGMA integrity_check` → `DbError::Corrupted` with guidance | F5.8 |
| Every list command paginated (cursor, max 500) | arch test over the policy registry (`returns_list = true` ⇒ `PageRequest` argument) | F5.8 |
| Indexes for sync tables | convention guard checks indexes on `owner_profile_id`, `visibility`, `updated_at`, `deleted_at` | F5.8 |
| Performance budgets on a 100 000-row reference dataset | `cargo bench` / criterion suite `benches/` + CI regression threshold 20 % | F5.8 (baseline), F20.3 (budgets) |
| Product-agnostic framework | `guard --only genericity`: word list (`family`, `keluarga`, `child`, `anak`, `cash`, `kas`, `household`, `rumah tangga`) outside examples, Notes sample and `Notes/` | F2.7 |
| Every public core item documented | `#![deny(missing_docs)]` in `caf-core` (ratchet allowed during F1–F10, zero from F11) | F1.7 |

## 2. Stage map

| Stage | Title | Main fixes | Depends on | Runner | Size |
|---|---|---|---|---|---|
| F0 | Baseline and evidence scaffolding | 7.3, 7.4, T-9 · S0.4 | — | any (x1-bench preferred) | S |
| F1 | Toolchain green and guard rails | S-2, 7.2, S-5 (CI) · S12.1 | F0 | any | M |
| F2 | Purge SSH/terminal/CATerm leftovers | 1.5–1.7, S-4, S-5 (files) · S1.2, S2.1–S2.5 | F1 | any | M |
| F3 | `app.toml` contract and codegen | 2.1–2.6, T-6 (schema), R-1 · S3.1–S3.4 | F2 | any | M |
| F4 | Vault hardening I | T-1, S-1, S-7 (vault), R-2, R-5, 3.3, 3.9, 3.13 · S6.1 | F3 | any | M |
| F5 | Data foundation | T-7, S-3, 3.5, 3.6, 4.1–4.9 · S5.1–S5.6 | F4 | any | L |
| F6 | Key hierarchy: DEK and recovery code | T-3, 3.4 · S6.1b | F5 | any | L |
| F7 | IPC contract and generated bindings | T-4, 5.12, 6.9, R-3 | F6 | any (+ VS-A) | M |
| F8 | Session, roles and capabilities | C-1 (a), T-2 (core), 3.9, 3.11, 3.14 (core) · S6.2–S6.5, S6.8 | F7 | any | L |
| F9 | Visibility and ownership | C-1 (b), 3.12, S-7 (notes) · S6.6 | F8 | any | M |
| F10 | Lock-screen UX: two levels, recovery, auto-lock | T-2 (UI), T-3 (UI), 3.8, 3.10, 3.14 · S6.7, S6.9 | F9 | any + owner smoke | L |
| F11 | Backup v2 and scheduled backup | T-5 · S5.7, S5.7b | F10 | any + owner smoke | L |
| F12 | Module switches, endpoints, Pro, feedback, crash, updater | 2.4, S-6, S-8, S-9 (capability), 6.10 · S8.1–S8.4, S5.8 | F11 | any | L |
| F13 | AI assistant, FR-7 complete | T-10, 6.2–6.8 · S7.1–S7.5 | F12 | any + owner check | L |
| F14 | Shell and design system | T-8, 5.4–5.10, R-4 · S4.1–S4.7 | F13 | any + owner visual | L |
| F15 | i18n completion and formatting | 5.3, 5.11 · S4.8 | F14 | any | M |
| F16 | Notes sample completion | FR-11, success criterion 3 · S9.1–S9.5 | F15 | any + owner smoke | M |
| F17 | Docs and open-source hygiene | FR-14, S-5 (docs), D-6 prep · S12.3, S12.4, S13.1–S13.2 | F16 | any | M |
| F18 | Generator `new-app` | T-6, 2.7–2.13, 7.6 · S10.1–S10.7, S12.2 | F17 | any (+ x1-bench) | L |
| F19 | Android | FR-13, S-9 · S11.1–S11.4 | F18 | x1-bench | M |
| F20 | Release readiness, QA re-audit and DoD | 7.5–7.7, NFR performance, PRD §9 · S14, S15 | F19 | x1-bench | M |

**Why this order differs from the draft.** Evidence and a green toolchain come first so every later gate can be proven. The `app.toml` contract (roles, modules, error prefix) comes before the capability checks that read it. The vault, migration runner and key hierarchy come before features that change schema or key material. The IPC contract is generated before the session work changes every command signature. All security work (F4–F10) precedes feature work.

## 3. Owner decisions (confirmed 4 October 2026)

The agent copies this table into the ledger. A decision changes only when the owner says so in writing; the agent records the date and the affected tasks.

| ID | Question | Decision | Affects |
|---|---|---|---|
| D-1 | Which `app.toml` schema is the contract? (audit §6 Q1) | PRD **v1.1** FR-1 in full, including the optional `[window]` section and the v1.1 sections (`[brand]` mandatory, `[rbac]`, `[pro]`, `[[integrations]]`, `[[jobs]]`, `[[entities]]`). F3 parses and validates all of them now so the contract does not change again in v0.2.0; sections whose features ship in v0.2.0 are only validated and code-generated. Current `[security]` fields become Settings defaults (D-7). Any further extension is proposed in `Notes/prd-amendments.md`, never silently added. | F3, F18 |
| D-2 | May the top role read other profiles' `private` rows? (audit §6 Q2; **revised 4 Oct 2026**) | **Yes, only the super role** (`[rbac].super_role`, default `super_admin`; PRD v1.1 FR-6). Every such read writes an audit event `PRIVATE_READ` (profile, entity, row count) and the UI marks the rows with the owner's name and a "private" badge. **No other role**, including `admin`, can read another profile's `private` rows; others see `private_summary` rows only as aggregates. | F8, F9, F11, F13 |
| D-3 | Where does the desktop launcher live? (audit §6 Q3) | `packaging/linux/<slug>.desktop` plus icon, generated from `app.toml` by xtask. CATerm snap/flatpak/docker-compose files are deleted. Snap/Flatpak packaging is out of scope for v0.1.0; Tauri bundles (deb/AppImage/rpm) remain. | F2, F18, F20 |
| D-4 | Pro scope for v0.1.0 (v0.2.0 extends it, see D-12) | Licensing client only: status, login, logout, register, start trial, forgot password, resend verification, account, revoke device, server availability. `app_id` = `[app].identifier` on every call; tested against a mock server. Framework `app.toml` ships `pro = false`. `ProTeamPanel` and the 9 team/sync calls are **removed** until Pro teams and sync ship. This deviates from the PRD §4 keep list and is recorded in `Notes/prd-amendments.md`. | F7, F12 |
| D-5 | Existing pre-release vaults (`vault.key` layout, Argon2(master) layout) | **No migration** (owner has no data to keep). On startup the core detects a legacy layout and returns `VaultError::LegacyFormat`; the UI explains that the data was created by a pre-release build and shows how to delete the data dir. The app never deletes it automatically. | F4, F5, F6, F10 |
| D-6 | Publication hygiene before the first public push | Purge `build/bin/caterm` (15 MB) from git history and move `Notes/` to a private repo (for example `caframework-notes`), so the audit is not public before the fixes are. The agent prepares the commands and a dry-run report in a temporary clone; **the owner executes**. | F17, F20 |
| D-7 | Security defaults (all configurable by the owner in Settings) | Profile lock after 5 min idle, vault lock after 15 min idle. Master password ≥ 12 characters. PIN 4–8 digits. Back-off for both: 5 free attempts, then 30 s doubling to a 15 min cap. 10 consecutive PIN failures (any profile) lock the vault. | F4, F8, F10 |
| D-9 | RBAC model (4 Oct 2026) | Permissions `<resource>:<action>` with wildcards; roles and permission sets from `[rbac]` in `app.toml`; generic default roles `super_admin`, `admin`, `member`, `viewer`. v0.1.0: core permission checks and read-only role display. v0.2.0: runtime role editing by the super role. | F3, F8, F10 |
| D-10 | Backup mandatory (4 Oct 2026) | `backup` cannot be switched off; `app.toml` validation rejects `modules.backup = false`. Per-entity data import/export ships in v0.2.0. | F3, F11 |
| D-11 | Branding mandatory (4 Oct 2026) | `[brand]` required; framework defaults: Cecep Saeful Azhar Hidayat, ST · https://www.cecepazhar.com · hi@cecepazhar.com · +6285220696117 (shown 0852 2069 6117). Shown in About, lock-screen footer, installer metadata, README/NOTICE/SECURITY contact, User-Agent. | F3, F12, F14, F17, F18 |
| D-12 | Pro package (4 Oct 2026) | v0.2.0: Pro always generated with Free/Pro feature gating via `[pro.features]`; payments on GCC/Mayar.id. v0.1.0 keeps D-4. | (v0.2.0) |
| D-13 | Quality bars (4 Oct 2026) | Stable, maintainable, scalable as measurable bars (§1.10, PRD §6). | all |
| D-8 | How is the agent run? | **One long session** that continues from stage to stage without waiting for approval, as long as each gate passes. It stops only for a STOP condition (`prompt-fix.md` §3). Because long sessions lose context, the ledger is the source of truth and the agent re-reads the current stage section of this file before starting each stage. This overrides `prompt-dev.md` §2.4/§8.2 ("stop at each gate") for this remediation only. | all |

## 4. Stages and tasks

---

### F0 — Baseline and evidence scaffolding

**Fixes:** 7.3, 7.4, T-9; confirms every audit *prediksi* · `task.md` S0.4 · **Runner:** any (x1-bench preferred, so VS-A runs too) · **Depends on:** — · **Size:** S

**No source code changes in F0.** Only `Notes/`, `guards/ratchet/` (baseline files) and the work branch are created.

- [ ] **F0.1 Work branch and pin.**
  **Do:** confirm `git status --porcelain` is empty; record `git rev-parse HEAD` (expected `7ec9e64…`; if different, record the drift and list commits since `7ec9e64` with `git log --oneline 7ec9e64..HEAD`); create `fix/remediation-v2` from HEAD. Record toolchain: `rustc -V`, `cargo -V`, `node -v`, `npm -v`, `cargo deny --version`, `gitleaks version` (install missing tools and record the install commands), OS and free disk (`df -h .`).
  **Accept:** branch exists; HEAD and tool versions recorded.
  **Verify:** `git branch --show-current` · `git log -1 --format='%H %s'`.

- [ ] **F0.2 Notes scaffolding.**
  **Do:** create `Notes/` files: copy `prd.md`, `task.md` (→ `Notes/task.md`, fixes 7.4), `prompt-dev.md`, this file (`Notes/TASK_FIX.md`), `prompt-fix.md`, and the audit (`Notes/audit/2026-10-02-audit-statis-caframework.md`). Create `Notes/fix-ledger.md` from `prompt-fix.md` §6.1 with all F0–F20 tasks as `TODO`, the §3 decision table, and an empty OWNER-VERIFY queue. Create `Notes/evidence/fix/` and `Notes/evidence/fix/logs/`. Create `Notes/prd-amendments.md` with entries for D-1 (`[window]`), D-4 (Pro teams removed) and D-5 (no legacy migration).
  **Accept:** all files exist and are committed; `Notes/task.md` is byte-identical to the owner's `task.md` (sha256 match).
  **Verify:** `ls -R Notes | head -50` · `sha256sum Notes/task.md`.

- [ ] **F0.3 Run the audit verification checklist (audit §5).**
  **Do:** in a throwaway clone (`git clone "$REPO_DIR" "$(mktemp -d)/caf-audit"`), run sections A, B, C, E, F, G (`cargo test -p caf-core ai::` only) and H of audit §5 verbatim, each into its own log. Section C.4 writes to `/tmp/caf-demo`; use a `mktemp -d` path instead and delete it afterwards. Section D (owner's real data dir) and the GUI checks are **not** run by the agent; add them to the OWNER-VERIFY queue (§1.5) with the exact commands. VS-A runs only on x1-bench.
  **Accept:** a table in `F0.md` lists every audit claim marked *prediksi* or BELUM DIVERIFIKASI with the observed result: `CONFIRMED`, `REFUTED` (with output), or `OWNER-VERIFY`. Minimum rows: clippy failure at `ai.rs:99/107`; 25 unregistered invokes; 113 autogenerated permission files; codegen reproducibility; PRD FR-1 example rejected; unknown key accepted; `new-app` output size and missing `.github`; identical updater pubkey; `CREATE TRIGGER` absent; `command_logs` absent; 0 `dark:` in layout/pages; `build/bin/caterm` > 1 MB; `git tag -l v0.1.0` empty; gitleaks result.
  **Verify:** the logs in `EVID/logs/F0-audit-*.log`.

- [ ] **F0.4 Baseline measurements.**
  **Do:** measure and record the baseline count for every ratchet in §1.3 using the audit's own commands where they exist (audit §5 F for invokes; `grep -c` for `indigo-`); `git ls-files | wc -l`; test count per crate (`cargo test --workspace --exclude caf-app -- --list 2>/dev/null | grep -c ': test$'`). Write the raw lists into `guards/ratchet/*.txt` with `# resolves-in:` tags (the guard that reads them arrives in F1).
  **Accept:** each ratchet file exists with a header line stating its baseline count and source command; counts match the logs.
  **Verify:** `wc -l guards/ratchet/*.txt`.

- [ ] **F0.5 Classify commit-message claims (T-9).**
  **Do:** for commits `1237508..7ec9e64` list each claim ("tests pass", "verified with E2E", "v0.1.0 release build", "desktop launcher"…) with status `UNSUPPORTED` / `CONTRADICTED` / `SUPPORTED`, based on F0.3 evidence.
  **Accept:** table in `F0.md`; no claim marked `SUPPORTED` without a log line.
  **Verify:** `git log --format='%h %s' 1237508^..7ec9e64`.

**Gate F0:** `Notes/` scaffolding committed; every prediction has an observed status; ratchet baselines recorded; first stage report sent (Bahasa Indonesia).

---

### F1 — Toolchain green and guard rails

**Fixes:** S-2, 7.2, S-5 (CI part), groundwork for 7.1 · `task.md` S12.1 (partial) · **Runner:** any · **Depends on:** F0 · **Size:** M

- [ ] **F1.1 Fix the clippy `unwrap_used` violations (S-2).**
  **Do:** in `CORE/ai.rs ~94-114` replace `Regex::new(..).unwrap()` with regexes compiled once via `std::sync::LazyLock` (or `OnceLock`) that return a typed error path; the patterns are constants, so add a unit test that every pattern compiles. Do not add `#[allow(clippy::unwrap_used)]`. Fix any other clippy finding the same way.
  **Accept:** `cargo clippy --workspace --all-targets --exclude caf-app -- -D warnings` exits 0; `git grep -n 'allow(clippy::unwrap_used)'` shows no new hits outside `#[cfg(test)]`.
  **Verify:** the clippy command; the grep.

- [ ] **F1.2 Format and dependency policy green.**
  **Do:** `cargo fmt --all`; run `cargo deny check`; fix advisories by upgrading, never by adding ignores without a ledger entry that states advisory ID, reason and expiry date.
  **Accept:** VS-R green.
  **Verify:** VS-R log.

- [ ] **F1.3 `caf-xtask guard` subcommand.**
  **Do:** add `cargo run -p caf-xtask -- guard [--only <name>]` that implements every ratchet in §1.3 plus three hard guards: (a) no tracked file > 1 MB and no tracked binary/DB (by extension and by NUL-byte sniffing), (b) no dev-machine absolute path outside `Notes/`, (c) every `#[tauri::command]` in `APP` is registered in `generate_handler!`, `build.rs` and the ACL (moves the existing arch check into one place; keep the arch test calling it). The invoke scanner must handle `invoke('x')`, `invoke<T>('x')`, `invoke("x", …)` and multi-line calls, and must also report invokes whose name is not a string literal. Output: one line per violation `<guard>: <file>:<line>: <detail>`, exit 1 on any failure.
  **Accept:** each guard has a fixture test under `crates/caf-xtask/tests/guard_*.rs` that proves it fails on a bad fixture and passes on a good one; running `guard` on the repo passes with the F0 ratchet files.
  **Verify:** `cargo test -p caf-xtask --test 'guard_*'` · `cargo run -p caf-xtask -- guard`.

- [ ] **F1.4 Hard-coded string and raw-sync-write baselines.**
  **Do:** implement the `hardcoded_strings` scanner (text nodes and `placeholder`/`title`/`aria-label` attribute literals in `.svelte` files outside `FE/lib/generated`) and the `raw_sync_writes` scanner (SQL write statements naming `notes`, `profiles` or any table created by a framework migration, outside `CORE/sync.rs` and `CORE/migrations/`); record baselines in the ratchet files.
  **Accept:** baselines recorded; fixture tests prove both scanners fail on a bad fixture.
  **Verify:** `cargo run -p caf-xtask -- guard --only hardcoded_strings` · `--only raw_sync_writes`.

- [ ] **F1.5 CI workflow.**
  **Do:** rewrite `.github/workflows/ci.yml` into jobs: `rust` (VS-R), `frontend` (VS-F), `guards` (VS-G), `secrets` (`gitleaks detect --source . --log-opts=--all`), `app` (ubuntu with `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf` → VS-A). Placeholders for `e2e` (filled in F18) must not exist as skipped jobs; add them when the test exists. Pin action versions by tag; cache cargo and npm.
  **Accept:** workflow file passes `actionlint` (install via the package manager or `go install`; if unavailable, record NOT VERIFIED and the owner command); if a remote and push rights exist (`prompt-fix.md` §0), a CI run on the work branch is linked with all jobs green; otherwise add an OWNER-VERIFY entry.
  **Verify:** `actionlint .github/workflows/ci.yml` · CI run URL or OWNER-VERIFY ID.

- [ ] **F1.6 Frontend test runner.**
  **Do:** add `vitest` (+ `@testing-library/svelte`, `jsdom`) and `npm run test` (`vitest run`). Add a smoke test for `errorText()` parsing `{ code, message, domain }` and a non-object error.
  **Accept:** `npm run test` passes with ≥ 2 tests; dependency licences recorded.
  **Verify:** VS-F log.

- [ ] **F1.7 Quality-bar tooling (§1.10, D-13).**
  **Do:** enable the stricter clippy set in `caf-core` and `caf-xtask` for non-test code (`unwrap_used`, `expect_used`, `panic`, `unreachable`, `indexing_slicing`) and fix or ratchet existing hits (`guards/ratchet/strict_lints.txt`, resolves by F5); add `guard --only size` (files ≤ 600 lines, functions ≤ 80 lines via `syn`/line scan, exceptions in `file_size_allowlist.txt`, shrink-only); add coverage measurement (`cargo llvm-cov --workspace --exclude caf-app --summary-only`, `vitest --coverage`) and record the baseline in `guards/ratchet/coverage.txt` (gate fails if coverage drops); add `missing_docs` as a ratchet; add `proptest` as a dev-dependency.
  **Accept:** each new guard has a negative fixture; coverage baseline recorded; CI runs the coverage step.
  **Verify:** `cargo clippy -p caf-core --lib -- -D warnings` · `cargo run -p caf-xtask -- guard --only size` · `cargo llvm-cov --workspace --exclude caf-app --summary-only`.

**Gate F1:** VS-R, VS-F, VS-G green; every guard has a passing negative fixture test; CI file valid; quality-bar baselines recorded.

---

### F2 — Purge SSH/terminal/CATerm leftovers

**Fixes:** 1.5, 1.6, 1.7, S-4, S-5 (files), D-3 (removal part) · `task.md` S1.2, S2.1–S2.5 · **Runner:** any (+ VS-A) · **Depends on:** F1 · **Size:** M

Work in small batches; after each batch run `cargo test --workspace --exclude caf-app` and `npm run check`. Grep references before deleting anything (`prompt-dev.md` §2.5).

- [ ] **F2.1 Stale Tauri permissions.**
  **Do:** delete `crates/caf-app/permissions/autogenerated/`; rebuild the app crate so `tauri-build` regenerates only current commands (VS-A).
  **Accept:** on x1-bench/CI `app`: number of generated permission files equals the number of registered commands (25 at this point) plus Tauri's default set, and none matches `ssh_|sftp_|tunnel|_host|_key|local_|remote_|watch|command_logs`. Without VS-A: OWNER-VERIFY.
  **Verify:** `ls crates/caf-app/permissions/autogenerated | wc -l` · `ls … | grep -E '<pattern>'` (empty).

- [ ] **F2.2 Frontend leftovers.**
  **Do:** delete `FE/lib/nav.ts`; remove shortcut groups terminal/sftp/sessions plus `isTerminalTarget` and `terminalKeyAction` from `shortcuts.ts`; remove `terminalTheme` (`theme.svelte.ts ~56`); remove `hosts` mode from `commandPalette.svelte.ts`; delete `OsIcon.svelte`, `static/hostinger-logo.jpg`, the `terminal` avatar preset, and `api/performance.ts` (with the `save_performance_prefs` invoke and any UI using it); remove SSH/terminal strings from `en.ts`/`id.ts`. Remove unused packages: 12× `@codemirror/*`, `autoprefixer`, `postcss` (confirm unused with `npx depcheck` and grep).
  **Accept:** `git grep -n -I -i -E 'ssh|sftp|\bscp\b|\bftp\b|webdav|xterm|terminal|tunnel' -- frontend/src` returns only entries listed in `forbidden_tokens_allowlist.txt`, each with a reason; `npm run check` 0 errors; `unregistered_invokes.txt` lost `save_performance_prefs`.
  **Verify:** the grep · VS-F · `cargo run -p caf-xtask -- guard`.

- [ ] **F2.3 Rust leftovers.**
  **Do:** remove terminal prefs (`CORE/prefs.rs ~21-24`: scrollback, inactive session) and their serde fields; rewrite SSH comments in `secret.rs ~1-7`, `window.rs ~40,79,149`, `crash.rs ~171`; replace the `CatermError` alias (`error.rs ~118`, `lib.rs ~39` and all users) with `CafError`/domain errors; remove orphan crates from caf-core (`notify`, `md-5`, `hmac`, `ed25519-dalek`, `machine-uid`, `tokio`, `async-trait`, `once_cell`, `rand`, `secrecy`) and caf-app (`async-trait`, `parking_lot`) **unless** a later stage in this plan needs them (`secrecy` is re-added with real use in F4; `rand` in F6). Use `cargo machete` (or `cargo udeps` on nightly) and grep to confirm.
  **Accept:** `git grep -c -I 'CatermError'` = 0; machete reports no unused dependency; VS-R green.
  **Verify:** the grep · `cargo machete` · VS-R.

- [ ] **F2.4 Packaging and root files (D-3).**
  **Do:** `git rm` `snap/`, `flatpak/`, `docker-compose.test.yml`, `task-v2.md`, `ledger-v2.md`, `agent-master-prompt.md`, `caterm-execution-prompts-v2-addendum.md`, `optimization.md`, `assets/logo-caterm*.png` (after confirming no reference), and `build/bin/caterm` (history purge is D-6, owner). Rename the release secret `CATERM_PRO_LICENSE_PUBKEY_V1` in `release.yml` to `CAF_PRO_LICENSE_PUBKEY_V1` (product prefix from `[app].code` after F3) and record the owner action "create the new secret name in GitHub" in OWNER-VERIFY. Add `dist/`, `crates/caf-app/gen/`, `build/` to `.gitignore`.
  **Accept:** `oversized_or_binary.txt` and `absolute_paths.txt` contain only header lines (0 violations) after F2.5; `git grep -I -i caterm` hits only `NOTICE`, `CHANGELOG.md` port notes, `Notes/` and entries in the allowlist.
  **Verify:** `git ls-files -z | xargs -0 du -b | awk '$1 > 1048576'` (empty) · the grep.

- [ ] **F2.5 Absolute paths in docs.**
  **Do:** replace the dev-machine paths in `docs/android-build-guide.md ~34` and `docs/android-signing.md ~11,28,34` with `$HOME`-relative or placeholder paths.
  **Accept:** `guard --only absolute_paths` passes with an empty ratchet.
  **Verify:** the guard output.

- [ ] **F2.6 Untracked CATerm artefacts (owner, Tier 2).**
  **Do:** add an OWNER-VERIFY entry asking the owner to delete the untracked local folders `dist/` (≈ 788 MB CATerm packages) and `crates/caf-app/gen/android` (≈ 940 MB CATerm Android project; regenerated in F19), with the commands `du -sh dist crates/caf-app/gen/android` before and `ls` after. The agent does not delete them.
  **Accept:** entry exists; F18's generator does not depend on these folders being deleted (it copies from `git ls-files`).
  **Verify:** ledger entry ID.

- [ ] **F2.7 Genericity guard (PRD §6, D10).**
  **Do:** add `guard --only genericity` with the word list in §1.10; replace product-specific wording in framework code, defaults and strings (for example "household master password" → "master password", family role names → generic roles from D-9); examples and the Notes sample are exempt by path.
  **Accept:** guard passes with an empty ratchet; negative fixture shown.
  **Verify:** `cargo run -p caf-xtask -- guard --only genericity`.

**Gate F2:** VS green; forbidden-token allowlist reviewed (each entry has a reason); `unregistered_invokes` ≤ 24; on x1-bench or OWNER-VERIFY, the app still launches to the lock screen.

---

### F3 — `app.toml` contract and codegen

**Fixes:** 2.1, 2.3, 2.4 (contract part), 2.5 (verify), 2.6, T-6 (schema), R-1 · `task.md` S3.1–S3.4 · D-1 · **Runner:** any · **Depends on:** F2 · **Size:** M

- [ ] **F3.1 Schema types (D-1).**
  **Do:** move the schema into `XT/config.rs` as Rust types matching PRD FR-1 exactly, every struct with `#[serde(deny_unknown_fields)]`:
  - `[app]`: `name`, `slug`, `code`, `identifier`, `accent`, `default_locale`, `locales`, `platforms`, `caframework_version` (optional in input, written by the generator).
  - `[window]` (optional): `width` (default 1200), `height` (800), `min_width` (900), `min_height` (600), `resizable` (true).
  - `[modules]`: `profiles`, `ai`, `pro`, `feedback`, `updater`, `crash`, `backup`, `support_page`; each optional, default `false`, so a product enables only what it needs.
  - `[brand]` (**required**, D-11): `developer`, `company` (optional), `website`, `email`, `phone` (E.164), `support_url` (optional), `copyright` (supports `{year}`), `logo`, `show_on_lock_screen` (default true).
  - `[rbac]` (D-9): `super_role`, `allow_runtime_roles` (default true), `[[rbac.roles]]` with `key`, `label_en`, `label_id`, `permissions`.
  - `[profiles]`: `pin_required_for`, `age_stages` (default `[]`, elements `{ key, min_age, max_age }`).
  - `[pro]` and `[pro.features]`, `[[integrations]]`, `[[jobs]]`, `[[entities]]` exactly as PRD v1.1 FR-1 (validated and code-generated now; features in v0.2.0).
  - `[endpoints]`: `gcc_base_url`, `updater_manifest_url` (default `""`).
  - `[[menus]]`: `key`, `label_en`, `label_id`, `icon`, `accent` (default `""`), `group`, `permission` (default `<key>:view`).
  Remove the old fields (`bundle_id`, `accent_color`, `error_prefix`, `version`, `description`, `*_enabled`, `[roles]`, `label`, `href`, `roles` on menus, `[security]`). `modules.backup` is not a valid key (D-10); `modules` gains `integrations` and `scheduler`. Map each `[security]` value to the D-7 Settings defaults and record the mapping in `F3.md`.
  **Accept:** the PRD v1.1 FR-1 example, copied verbatim into `crates/caf-xtask/tests/fixtures/prd-fr1.toml`, parses successfully; a `proptest` suite feeds random mutations of it to the parser and validator without a panic (§1.10).
  **Verify:** `cargo test -p caf-xtask --test config`.

- [ ] **F3.2 Validation with readable errors.**
  **Do:** implement `validate()` returning all errors at once, each with the TOML path and line/column (use `toml` spans). Rules:
  - `slug` `^[a-z][a-z0-9-]{1,30}$`, not `caf`/`caframework` unless the framework itself; `code` `^[A-Z]{2,6}$`; `identifier` reverse-DNS `^[a-z][a-z0-9]*(\.[a-z][a-z0-9-]*){2,}$`;
  - `accent` and every non-empty menu `accent` ∈ {violet, sky, emerald, amber, rose, cyan, indigo, teal, orange, pink};
  - `locales` non-empty, ⊆ {id, en}, unique; `default_locale` ∈ `locales`; `platforms` non-empty ⊆ {linux, windows, macos, android};
  - `[brand]`: `developer` non-empty; `website` and `support_url` `https://…`; `email` valid; `phone` E.164 `^\+[1-9][0-9]{7,14}$`; `logo` path exists; missing `[brand]` → error "brand is mandatory";
  - `[rbac]`: role keys non-empty, unique, `^[a-z][a-z0-9_]{0,31}$`; `super_role` ∈ roles and holds exactly `["*"]`; every permission matches `^(\*|[a-z][a-z0-9_.-]*:(\*|view|create|update|delete|export|import|manage|run))$` and names a known resource (menus, entities, integrations, jobs, system areas); `pin_required_for` ⊆ roles; age stages non-overlapping with `min_age ≤ max_age`;
  - `modules.backup` present → error (backup is mandatory); integration, job and entity keys unique and `^[a-z][a-z0-9_.-]{0,47}$`; cron expressions parse; `[pro.features]` values ∈ {free, pro};
  - endpoints empty or `https://…`; `http://localhost`/`127.0.0.1` allowed only with a warning;
  - ≥ 1 menu; menu `key` `^[a-z][a-z0-9-]{0,31}$`, unique, not reserved (`settings`, `lock`, `_dev`, `api`); labels non-empty; `icon` matches SVG path grammar `^[MmLlHhVvCcSsQqTtAaZz0-9 ,.\-]+$` and ≤ 2000 chars; `group` `^[a-z][a-z0-9_]{0,31}$`; menu `permission` refers to a known resource.
  **Accept:** one test per rule (valid and invalid), at least 40 cases, including "unknown key rejected" (audit §5 C.3) and "PRD example accepted" (audit §5 C.2); error messages contain the field path.
  **Verify:** `cargo test -p caf-xtask --test config -- --nocapture` (paste failures shown by the invalid cases).

- [ ] **F3.3 Framework `app.toml`.**
  **Do:** rewrite the repo `app.toml`: `name = "CAFramework"`, `slug = "caframework"`, `code = "CAF"`, `identifier = "com.fathforce.caframework"`, current accent, `locales = ["id","en"]`, all four platforms, `[brand]` with the D-11 defaults, modules `profiles/ai/feedback/updater/crash = true`, `pro = false` (D-4; v0.2.0 switches it on), `integrations/scheduler = false` until v0.2.0, `support_page = false`, the generic roles of D-9 with the PRD example permissions, empty endpoints, entity and menu `notes`.
  **Accept:** `cargo run -p caf-xtask -- validate --config app.toml` exits 0.
  **Verify:** command output.

- [ ] **F3.4 Codegen outputs and `--check`.**
  **Do:** `codegen` writes, each with the header `@generated by caf-xtask codegen from app.toml — do not edit`:
  - `FE/lib/generated/app.ts`: `APP_CONFIG` (name, slug, code, identifier, accent, locales, defaultLocale, platforms, modules, superRole, pinRequiredFor, flags `hasGccEndpoint`/`hasUpdaterEndpoint`, `caframeworkVersion`);
  - `FE/lib/generated/navItems.generated.ts`: key, href `/m/<key>`, label key `menu.<key>`, icon path, accent, group, permission;
  - `FE/lib/generated/menuLabels.ts`: `{ en: {…}, id: {…} }` merged into the dictionaries;
  - `CORE/generated/app_config.rs`: `macro_rules! error_prefix { () => { "CAF" } }` (so `concat!` keeps working), `MODULES`, `PIN_REQUIRED_FOR`, `GCC_BASE_URL: Option<&str>`, `UPDATER_MANIFEST_URL: Option<&str>`, `APP_IDENTIFIER`, `APP_SLUG`;
  - `FE/lib/generated/brand.ts` and `CORE/generated/brand.rs` (all `[brand]` fields, `{year}` resolved at runtime);
  - `FE/lib/generated/permissions.ts` and `CORE/generated/permissions.rs`: resource catalogue, actions, default roles with permission sets, `SUPER_ROLE`;
  - patches to `crates/caf-app/tauri.conf.json`: `productName`, `identifier`, window size from `[window]`, updater endpoints from `updater_manifest_url` (updater plugin config removed when empty), `bundle.publisher`, `bundle.copyright`, `bundle.homepage`, `bundle.linux.deb.maintainer`-style fields from `[brand]` where Tauri supports them.
  `codegen --check` regenerates in memory and exits 1 on any diff. `--config` resolves output paths from the repo root, never from the config file's folder (audit §5 C.3 note).
  Replace the hard-coded `concat!("CAF-", …)` in `CORE/error.rs ~22,24` with `concat!(error_prefix!(), "-", …)`.
  **Accept:** running codegen twice gives `git diff --exit-code` = 0; changing `name`, `accent` or a menu in a temp copy of `app.toml` changes the generated files accordingly (show the diff); `git grep -n '"CAF-"' -- crates` returns only the generated file and tests.
  **Verify:** `cargo run -p caf-xtask -- codegen && cargo run -p caf-xtask -- codegen && git diff --exit-code` · `cargo run -p caf-xtask -- codegen --check`.

- [ ] **F3.5 Icon pipeline (S3.4).**
  **Do:** single source `assets/logo.svg` → `cargo tauri icon assets/logo.svg` produces the Tauri icon set; document the command in `docs/creating-an-app.md` (stub created now, finished in F18).
  **Accept:** icon files listed; on x1-bench/CI `app` the bundle config references them. Without VS-A: OWNER-VERIFY.
  **Verify:** `ls crates/caf-app/icons`.

- [ ] **F3.6 Portable data dir (R-1).**
  **Do:** portable mode resolves `<dir of the executable>/<slug>-data` instead of CWD-relative `./caframework-data` (`CORE/paths.rs ~57`); data dir name comes from `APP_SLUG`; env override renamed to `<CODE>_DATA_DIR` with `CAF_DATA_DIR` as the framework value.
  **Accept:** unit test with an injected executable path; no CWD dependence.
  **Verify:** `cargo test -p caf-core paths::`.

**Gate F3:** VS green including `codegen --check`; PRD example parses; ≥ 25 validation tests.

---

### F4 — Vault hardening I

**Fixes:** T-1, S-1, S-7 (vault part), R-2, R-5 (vault), 3.3, 3.9 (master password), 3.13 · `task.md` S6.1 · D-5, D-7 · **Runner:** any · **Depends on:** F3 · **Size:** M

- [ ] **F4.1 Remove the plaintext `vault.key` fallback (T-1).**
  **Do:** delete `load_or_create_local_key` (`CORE/vault.rs ~206-221`) and every call; `ensure_unlocked_key()` (`~34-42`) returns `VaultError::Locked` when no key is loaded. Any DB access before setup returns `VaultError::NotInitialized`.
  **Accept:** test `locked_vault_returns_locked` asserts the error code equals the registry code of `VaultError::Locked` (expected `CAF-VAULT-001`); test `vault_key_file_never_created` runs setup → lock → data command → unlock and asserts no file named `vault.key` exists in the temp data dir at any step; `git grep -n 'vault.key' -- crates` only hits the legacy detector and tests.
  **Verify:** `cargo test -p caf-core vault::` · the grep.

- [ ] **F4.2 Legacy layout detection (D-5).**
  **Do:** on startup, if the data dir contains `vault.key`, or contains a DB without the F5 `schema_version` layout, `vault_status` returns `LegacyFormat` and every data command returns `VaultError::LegacyFormat`. Nothing is deleted or migrated.
  **Accept:** tests with fixture data dirs for both legacy shapes; the files are untouched afterwards (sha256 before = after).
  **Verify:** `cargo test -p caf-core vault::legacy`.

- [ ] **F4.3 Test vault helper and real unlock in all DB tests.**
  **Do:** add `#[cfg(test)] fn test_vault() -> TestVault` that creates a temp data dir, initialises with a fixed test password and unlocks; KDF parameters come from a `KdfParams` value so tests can use `m = 8 MiB, t = 1, p = 1`, while a separate test asserts production params `m = 64 MiB, t = 3, p = 4`. Rewrite the DB tests in `profiles.rs ~279`, `notes.rs ~232`, `ai.rs ~250` to use it.
  **Accept:** `git grep -n 'ensure_unlocked_key' -- crates` shows no test relying on an implicit key; all tests pass.
  **Verify:** `cargo test -p caf-core`.

- [ ] **F4.4 Zeroization (S-1).**
  **Do:** keep the active key as `Zeroizing<[u8; 32]>` inside a `secrecy::SecretBox` (or equivalent) in core state; on lock, zeroize in place (`if let Some(k) = guard.as_mut() { k.zeroize() }`) and then set `None`; build every `PRAGMA key`/`rekey` string inside `Zeroizing<String>`; command arguments carrying a password or PIN are converted to `SecretString` at the first line of the command and never cloned into `String`. Document in `docs/security.md` (stub) that serde/IPC buffers outside our control may still hold copies.
  **Accept:** `git grep -n -E 'format!\("PRAGMA (re)?key' -- crates` shows only `Zeroizing` construction; a unit test drives lock and asserts, via a test-only accessor, that the stored key bytes are all zero before release; `secrecy` is used (machete clean).
  **Verify:** `cargo test -p caf-core vault::zeroize` · the grep.

- [ ] **F4.5 Master-password back-off in core (3.9, D-7).**
  **Do:** the failure counter cannot live inside the encrypted vault (it is locked when the password is wrong). Store `auth_state.json` (0600) next to the vault with `{ failures, next_allowed_at_ms }`, keep an in-memory counter for the process lifetime, and use the maximum of both, so deleting the file while the app runs does not reset the counter. Policy (D-7): 5 free attempts, then 30 s doubling up to 15 min; success resets. Calls during back-off return `VaultError::LockoutActive { retry_after_ms }` without running Argon2. Inject a `Clock` trait for tests. Document the limitation: back-off slows guessing through the app; offline guessing is limited by Argon2id, not by this counter.
  **Accept:** tests with a fake clock: attempts 1–5 immediate; 6th returns `LockoutActive` with 30 000 ms; doubling sequence capped at 900 000 ms; reset on success; counter survives deleting the file mid-process; `Argon2` not invoked during back-off (counter in a test double).
  **Verify:** `cargo test -p caf-core vault::backoff`.

- [ ] **F4.6 Remove the localStorage lockout.**
  **Do:** delete the lockout logic in `LockScreen.svelte ~30-84`; show `retry_after_ms` from the core error as a countdown.
  **Accept:** `git grep -n -i -E 'lockout|attempt' -- frontend/src` shows no `localStorage` use; `npm run check` green.
  **Verify:** the grep.

- [ ] **F4.7 File permissions (R-2).**
  **Do:** create the data dir `0700` and every vault file (`vault_salt.bin`, `vault_canary.bin`, `device_id.txt`, `auth_state.json`, DB, later keyring and index) `0600` on Unix via one helper `paths::write_private_atomic()` (temp file, `fsync`, rename, directory `fsync`; §1.10), and a guard forbids `std::fs::write` in `caf-core` outside it; on Windows rely on the user profile ACL and say so in `docs/security.md`.
  **Accept:** test lists every file in a fresh data dir and asserts the mode.
  **Verify:** `cargo test -p caf-core paths::permissions`.

- [ ] **F4.8 Specific error variants (R-5, vault).**
  **Do:** add or use `VaultError::{Locked, NotInitialized, AlreadyInitialized, WrongPassword, LockoutActive, LegacyFormat, WeakPassword, Corrupted}`; replace every `Generic` return in `vault.rs`/`db.rs` for these cases; enforce master password ≥ 12 characters (D-7) with `WeakPassword`. Regenerate the registry and `docs/error-codes.md`.
  **Accept:** `git grep -n 'Generic' -- crates/caf-core/src/vault.rs crates/caf-core/src/db.rs` returns 0 hits on expected-failure paths (list any remaining with a reason); golden test green; a test asserts "wrong password" and "corrupted DB" produce different codes.
  **Verify:** `cargo test -p caf-core --test error_codes_unique` · the grep.

- [ ] **F4.9 CLI secrets (S-7, vault part).**
  **Do:** `caf-cli vault unlock` reads the password from a TTY prompt (`rpassword`) or from `--password-stdin`; remove the positional password argument. `profile add --pin` is removed in F8 (PIN prompt).
  **Accept:** `caf-cli vault unlock --help` shows no password argument; an integration test pipes the password through stdin.
  **Verify:** `cargo run -p caf-cli -- vault unlock --help` · `cargo test -p caf-cli`.

**Gate F4:** VS green; `vault.key` cannot be created; locked vault yields the `Locked` code; back-off proven with a fake clock.

---

### F5 — Data foundation

**Fixes:** T-7, S-3, 3.5, 3.6, 4.1, 4.3, 4.4, 4.6, 4.7, 4.8, 4.9 · `task.md` S5.1–S5.6 · **Runner:** any · **Depends on:** F4 · **Size:** L

- [ ] **F5.1 Migration runner (3.5).**
  **Do:** `CORE/migrations/` holds numbered SQL files embedded with `include_str!` (framework range `0001–0999`, product range `1000+`, sample Notes `0500–0599` so the generator can drop them). Table `schema_version(version INTEGER PRIMARY KEY, name TEXT NOT NULL, checksum TEXT NOT NULL, applied_at INTEGER NOT NULL)`. Each migration runs in its own transaction; a checksum mismatch returns `DbError::MigrationChecksum`; a DB newer than the code returns `DbError::SchemaTooNew`. Remove `CREATE TABLE IF NOT EXISTS` bootstrapping from `db.rs ~69-132`. Because of D-5 the first migration is a clean baseline, not an upgrade from the TEXT-timestamp schema.
  **Accept:** tests: empty DB → latest; latest → re-run is a no-op; edited migration → checksum error; `schema_version` higher than known → `SchemaTooNew`; failed migration leaves version unchanged.
  **Verify:** `cargo test -p caf-core migrations::`.

- [ ] **F5.2 Sync-ready schema (4.1, 4.3, S-3).**
  **Do:** baseline migration creates `profiles` and (sample range) `notes` with `id TEXT PRIMARY KEY` (UUIDv7), `created_at INTEGER NOT NULL`, `updated_at INTEGER NOT NULL` (unix ms UTC from the injected `Clock`), `deleted_at INTEGER NULL`, `rev INTEGER NOT NULL DEFAULT 1`, `origin_device_id TEXT NOT NULL`, `owner_profile_id TEXT NOT NULL`, `visibility TEXT NOT NULL CHECK (visibility IN ('shared','private_summary','private'))`. Create `change_log(seq INTEGER PRIMARY KEY AUTOINCREMENT, entity TEXT NOT NULL, entity_id TEXT NOT NULL, op TEXT NOT NULL CHECK (op IN ('insert','update','delete')), rev INTEGER NOT NULL, at INTEGER NOT NULL, profile_id TEXT, device_id TEXT NOT NULL)` exactly as PRD FR-5 names it. `app_kv` stays a key/value table and is listed as an explicit exclusion with a reason.
  **Accept:** `PRAGMA table_info` dump of every table pasted in `F5.md`; no `TEXT` timestamp and no `datetime('now')` left (`git grep -n "datetime('now')" -- crates` empty).
  **Verify:** test `schema_dump` printing `sqlite_master` + `pragma_table_info`.

- [ ] **F5.3 Central write helper (4.4, 4.6, 4.7).**
  **Do:** implement `CORE/sync.rs` with `insert`, `update` (rev + 1, `updated_at`), `soft_delete` (`deleted_at`, rev + 1) that write the row and its `change_log` entry in the **same** transaction, taking `&Transaction`, `SyncCtx { profile_id, device_id, clock }`. Move `notes.rs` and `profiles.rs` writes onto it, so `save_profile` and `delete_profile` log and bump `rev`. Decision recorded in `F5.md`: helper over SQLite triggers, because triggers cannot see the acting profile and device without a side table. `raw_sync_writes` ratchet goes to 0.
  **Accept:** tests per table: insert → rev 1 + one `insert` log row; update → rev 2 + `update` row; soft delete → rev 3 + `delete` row, `deleted_at` set; forced error after the row write → rollback leaves neither the row change nor a log row (4.9). `guard --only raw_sync_writes` passes with an empty ratchet.
  **Verify:** `cargo test -p caf-core sync::` · the guard.

- [ ] **F5.4 Convention guard test (4.8).**
  **Do:** `crates/caf-core/tests/conventions.rs` runs all migrations on a temp vault, reads `sqlite_master` and `pragma_table_info`, and fails if any table outside the exclusion list (`schema_version`, `change_log`, `app_kv`, `audit_events`, `pin_attempts`, `sqlite_sequence`) lacks a required column, has the wrong type or nullability, or lacks the `visibility` CHECK. The exclusion list is a constant with a reason per entry. Include a permanent negative test that applies a fixture migration with a table missing `rev` and asserts the guard reports it.
  **Accept:** both tests pass; negative test output pasted.
  **Verify:** `cargo test -p caf-core --test conventions -- --nocapture`.

- [ ] **F5.5 Plain-header guard (3.6).**
  **Do:** test creates a vault, writes a profile and a note, drops all handles, reads the first 16 bytes of the DB file and asserts they are not `SQLite format 3\0`; also asserts a known note title is not found anywhere in the raw file bytes.
  **Accept:** test passes; hex dump of the first 16 bytes in evidence.
  **Verify:** `cargo test -p caf-core --test plain_header -- --nocapture`.

- [ ] **F5.6 Audit events table (S-3).**
  **Do:** replace the never-created `command_logs` (`CORE/audit.rs ~46-50`, column `host_id`) with migration-created `audit_events(seq INTEGER PRIMARY KEY AUTOINCREMENT, at INTEGER NOT NULL, event TEXT NOT NULL, profile_id TEXT, detail TEXT)`; `log_event` returns an error that callers log instead of ignoring it. Events: `VAULT_INIT`, `VAULT_UNLOCK`, `VAULT_LOCK`, `VAULT_PASSWORD_CHANGED`, `RECOVERY_REGENERATED`, `PROFILE_CREATED/UPDATED/DELETED`, `PIN_RESET`, `BACKUP_EXPORTED/IMPORTED`. Details never contain secrets.
  **Accept:** test performs unlock + profile create and reads two audit rows; `git grep -n -E 'command_logs|host_id' -- crates` empty.
  **Verify:** `cargo test -p caf-core audit::` · the grep.

- [ ] **F5.7 Device id (4.10, regression guard).**
  **Do:** keep `device_id.txt` behaviour, written with `write_private_atomic()`.
  **Accept:** existing test still passes; file mode `0600`.
  **Verify:** `cargo test -p caf-core paths::device_id`.

- [ ] **F5.8 Integrity, pagination, indexes, benchmark baseline (§1.10).**
  **Do:** run `PRAGMA integrity_check` on unlock (`DbError::Corrupted` with guidance); add `PageRequest { cursor, limit ≤ 500 }` / `Page<T> { items, next_cursor }` and use it in every list function (keyset pagination on `(updated_at, id)`); add a `returns_list` flag to the policy registry with an arch test that such commands take `PageRequest`; create indexes on `owner_profile_id`, `visibility`, `updated_at`, `deleted_at` for every sync table and extend the convention guard to check them; add a criterion bench suite `crates/caf-core/benches/` with a generator for a 100 000-row reference dataset (notes) and benches for list page, search, insert batch; record the baseline.
  **Accept:** tests for corruption detection (truncated DB file) and pagination (stable order, no duplicates across pages, limit enforced); guard negative case for a missing index; bench baseline table in `F5.md`.
  **Verify:** `cargo test -p caf-core db::integrity pagination::` · `cargo bench -p caf-core -- --save-baseline f5`.

**Gate F5:** VS green; convention guard and plain-header test pass and their negative cases are shown; no write to a sync table bypasses `sync.rs`.

---

### F6 — Key hierarchy: DEK and recovery code

**Fixes:** T-3, 3.4, the non-atomic password change (T-3), salt reuse · `task.md` S6.1b · D-5 · **Runner:** any · **Depends on:** F5 · **Size:** L

Design (record as `docs/adr/0001-key-hierarchy.md`):

```
DEK  = 32 random bytes (OsRng)                 -> SQLCipher raw key
BK   = 32 random bytes (OsRng)                 -> backup key for scheduled backups (F11)
KEK_pw = Argon2id(master_password, salt_pw, m=64MiB, t=3, p=4)
KEK_rc = HKDF-SHA256(ikm = 256-bit BIP-39 entropy, salt_rc, info = "caf.kek.recovery.v1")
keyring.v1.json (0600):
  { format: 1, vault_id, kdf: {alg, m, t, p}, 
    password_slot: { salt_pw, nonce, ct = AEAD(KEK_pw, DEK||BK, aad = "caf-keyring-v1|password|" + vault_id) },
    recovery_slot: { salt_rc, generation, nonce, ct = AEAD(KEK_rc, DEK||BK, aad = "caf-keyring-v1|recovery|" + vault_id) } }
AEAD = AES-256-GCM (already a dependency), 96-bit random nonce per wrap
```

- [ ] **F6.1 Keyring module.**
  **Do:** implement `CORE/keyring.rs`: create, unwrap with password, unwrap with recovery code, rewrap password slot, replace recovery slot. Writes are atomic: write `keyring.v1.json.tmp`, `fsync`, rename, `fsync` the directory. The old canary file is removed from the design (the AEAD tag authenticates the password). A data dir with the old Argon2-direct layout (`vault_salt.bin` + `vault_canary.bin`, no keyring) is reported as `LegacyFormat` (D-5).
  **Accept:** tests: round trip with password and with recovery code; wrong password → `WrongPassword` (not `Corrupted`); flipping any ciphertext, nonce or AAD byte → `Corrupted`; truncated or malformed JSON → `Corrupted`, never a panic; fresh salts and nonces on every rewrap (two rewraps differ).
  **Verify:** `cargo test -p caf-core keyring::`.

- [ ] **F6.2 Setup with recovery code and confirmation.**
  **Do:** `initialize_vault(password)` creates DEK, BK and the keyring, opens the DB with DEK, and returns 24 BIP-39 English words once (in a `Zeroizing` value). The vault stays in state `RecoveryUnconfirmed`, in which every data command returns `VaultError::RecoveryUnconfirmed`, until `confirm_recovery_words(positions, words)` succeeds for 3 random positions chosen by the core. Words and entropy are never written anywhere. Add the `bip39` crate (licence check).
  **Accept:** tests: data command before confirmation fails with the specific code; wrong words fail; correct words unlock normal use; after setup, a byte scan of every file in the data dir finds none of the 24 words, the entropy hex, the DEK hex or the password.
  **Verify:** `cargo test -p caf-core keyring::setup -- --nocapture` (paste the scan summary, not the words).

- [ ] **F6.3 Change master password without rekey.**
  **Do:** `change_master_password(old, new)` verifies `old` by unwrapping, rewraps the password slot with a fresh salt, atomically replaces the keyring; no `PRAGMA rekey` (remove `vault.rs ~189-195`). Enforce D-7 length on `new`.
  **Accept:** tests: DB file sha256 identical before and after; old password fails, new works; recovery code still works; an injected failure before the rename leaves the old password working (no lock-out).
  **Verify:** `cargo test -p caf-core keyring::change_password`.

- [ ] **F6.4 Unlock with recovery code and reset password.**
  **Do:** `unlock_with_recovery(words)` unwraps via the recovery slot, then requires `set_new_master_password(new)` before any data command (state `PasswordResetRequired`).
  **Accept:** tests: recovery unlock → data command fails until new password is set → new password works → old password fails.
  **Verify:** `cargo test -p caf-core keyring::recovery_unlock`.

- [ ] **F6.5 Regenerate recovery code (revokes the old one).**
  **Do:** `regenerate_recovery_code()` (requires `security:manage` after F8; vault must be unlocked) creates new entropy, replaces the recovery slot, increments `generation`, returns the new words once, and requires confirmation as in F6.2.
  **Accept:** test: old words fail with `WrongRecoveryCode`, new words work, `generation` incremented.
  **Verify:** `cargo test -p caf-core keyring::regenerate`.

- [ ] **F6.6 Brute-force protection for recovery attempts.**
  **Do:** recovery attempts share the master-password back-off counter (F4.5).
  **Accept:** test: 6th wrong recovery attempt returns `LockoutActive`.
  **Verify:** `cargo test -p caf-core keyring::recovery_backoff`.

**Gate F6:** VS green; ADR committed; no plaintext key material on disk (scan output); password change proven not to rekey.

---

### F7 — IPC contract and generated bindings

**Fixes:** T-4, 5.12, 6.9, R-3, the `never_again` bug · **Runner:** any (+ VS-A for F7.6) · **Depends on:** F6 · **Size:** M

- [ ] **F7.1 Binding generation decision.**
  **Do:** default approach: `ts-rs` for every type crossing IPC, plus an xtask step that parses `APP/commands.rs` with `syn` and generates `FE/lib/generated/commands.ts` with one typed function per command, using Tauri 2's camelCase argument names and the Rust return type. Try `tauri-specta` only if it builds against the pinned Tauri version without RC-only features; record the choice and reasons in `docs/adr/0003-ipc-bindings.md`.
  **Accept:** ADR committed; `codegen --check` covers the generated bindings.
  **Verify:** `cargo run -p caf-xtask -- codegen --check`.

- [ ] **F7.2 Wrappers use only generated bindings.**
  **Do:** rewrite `FE/lib/api/*.ts` to call the generated functions; remove every direct `invoke(`. Fix the audited mismatches through the generated types: `change_master_password` (`old`/`new` → now `{ oldPassword, newPassword }` after renaming the Rust args to `old_password`/`new_password`), `export_encrypted_backup`/`import_encrypted_backup` (replaced in F11; until then, base64 `String` on both sides), `dismiss_crash_report` (`id` + `never_again` honoured in core, not hard-coded `false` at `commands.rs ~163`), `ai_chat` → `AiChatResponse { message, redactions_applied, tokens_used }`, `AiSettings` exactly as Rust, `Profile` with `avatar` and `has_pin` (never `pin_hash`).
  **Accept:** `invoke_outside_api` ratchet = 0; `npm run check` green; `git grep -n -E "avatar_url|pin_hash|guardrails_enabled|privacy_mode" -- frontend/src` empty; `AiChatPanel` renders `.message` (6.9).
  **Verify:** guard output · the grep · VS-F.

- [ ] **F7.3 Command policy registry.**
  **Do:** add `CORE/api/policy.rs`: every command name maps to one class: `Public` (works while locked: `vault_status`, `initialize_vault`, `unlock_vault`, `unlock_with_recovery`, `profile_index`, `app_info`, `reset_vault_locked`), `Unlocked` (vault open, no profile session needed: `select_profile`, `confirm_recovery_words`, `set_new_master_password`), `Session` (profile session required), `Requires(permission)` (RBAC permission, D-9; replaces the owner-only class). Commands carry no `caller_profile_id`/`is_owner` arguments from here on (removed in F8). An arch test fails if a registered command has no policy entry or a policy entry has no command.
  **Accept:** arch test passes and has a negative fixture.
  **Verify:** `cargo test -p caf-core --test arch`.

- [ ] **F7.4 Resolve the 25 unregistered invokes.**
  **Do:** decide each name and record the decision table in `F7.md`:
  | Name(s) | Decision |
  |---|---|
  | `is_vault_unlocked` (R-3) | replace with `vault_status` |
  | `export_vault_backup`, `import_vault_backup` | point the Settings UI to the registered `export_encrypted_backup`/`import_encrypted_backup` through generated bindings now; both are replaced by the F11 commands |
  | `open_external_url` | register; allows only `https://` URLs listed in `FE/lib/appInfo.ts` or the app's own endpoints |
  | `save_performance_prefs` | removed in F2 |
  | `submit_crash_report` | implement in F12 (needs endpoint); ratchet entry until then |
  | `pro_status`, `pro_login`, `pro_logout`, `pro_register`, `pro_start_trial`, `pro_forgot_password`, `pro_resend_verification`, `pro_account`, `pro_revoke_device`, `pro_server_available` | implement in F12 (D-4); ratchet entries until then |
  | `pro_sync`, `pro_commit_pending`, `pro_team`, `pro_team_accept`, `pro_team_cancel_invite`, `pro_team_decline`, `pro_team_invite`, `pro_team_leave`, `pro_team_remove_member` | delete with `ProTeamPanel` (D-4) |
  **Accept:** `unregistered_invokes.txt` contains only the 11 F12 entries, each tagged `resolves-in: F12`.
  **Verify:** `cargo run -p caf-xtask -- guard --only unregistered_invokes`.

- [ ] **F7.5 Frontend binding tests.**
  **Do:** vitest tests mock `@tauri-apps/api/core` `invoke` and assert, for every wrapper, the exact command name and argument object.
  **Accept:** one test per wrapper; tests pass.
  **Verify:** `npm run test`.

- [ ] **F7.6 IPC contract test (app crate).**
  **Do:** in `crates/caf-app/tests/ipc_contract.rs`, build the app with `tauri::test::mock_builder()` and the real handler, then for every command send the JSON argument object produced from the generated TS fixture (written by codegen to `crates/caf-app/tests/fixtures/ipc_args.json`) and assert the response is not an argument-deserialisation error.
  **Accept:** test passes in VS-A; without VS-A, OWNER-VERIFY entry with the command.
  **Verify:** `cargo test -p caf-app --test ipc_contract`.

**Gate F7:** VS green; no direct `invoke` outside generated code; every command has a policy class; parameter and type mismatches from T-4 cannot recur (tests).

---

### F8 — Session, roles and capabilities

**Fixes:** C-1 (identity and capability part), T-2 (core part), 3.9 (PIN), 3.11, 3.14 (core part) · `task.md` S6.2, S6.3, S6.4 (core), S6.5, S6.8 · D-2, D-7 · **Runner:** any · **Depends on:** F7 · **Size:** L

- [ ] **F8.1 Core session state.**
  **Do:** add `CORE/session.rs` with `Session { profile_id, role, started_at, last_activity_at }` held in the core `AppCore` (managed by Tauri as state; no global mutable statics for identity). Only `select_profile(profile_id, pin)` sets it, and only after the vault is unlocked and the PIN check passes. `lock_vault` and `lock_profile` clear it. Remove `caller_profile_id`/`is_owner` from every command signature and every core function (`APP/commands.rs ~88-121`).
  **Accept:** `git grep -n -E 'caller_profile_id|callerProfileId|is_owner|isOwner' -- crates frontend/src` is empty; command bodies stay within the arch test's line limit.
  **Verify:** the grep · `cargo test -p caf-core --test arch`.

- [ ] **F8.2 Policy enforcement wrapper.**
  **Do:** every command runs through `core.guarded(Policy, |ctx| …)`, which checks: vault state (`Locked`, `RecoveryUnconfirmed`, `PasswordResetRequired`), session presence (`AuthError::NoSession`), capability (`AuthError::PermissionDenied`), and module enabled (F12), then touches `last_activity_at`. The command layer only parses arguments and calls `guarded`.
  **Accept:** unit tests for each rejection path with distinct codes.
  **Verify:** `cargo test -p caf-core session::guard`.

- [ ] **F8.3 RBAC permission checks (D-9).**
  **Do:** `CORE/rbac.rs`: `Permission { resource, action }`, wildcard matching (`*`, `<resource>:*`), role → permission set loaded from the generated defaults and stored in vault tables `roles(key, label_en, label_id, is_builtin, …sync columns)` and `role_permissions(role_key, permission, …sync columns)` (seeded by migration; runtime editing UI is v0.2.0 G1). Every command's policy entry names the permission it needs (`OwnerOnly` becomes `Requires(permission)`); system permissions: `users:view|create|update|delete|manage`, `roles:view|manage`, `settings:manage`, `backup:export|import|manage`, `security:manage` (master password, recovery code, timers, reset), `audit:view`. Rules: a profile may edit its own name, avatar and PIN (current PIN required) without extra permission; creating profiles, changing roles and editing other profiles need `users:*`; the super role always holds `*` and at least one profile must keep it (cannot delete or demote the last one); assigned roles must exist.
  **Accept:** table-driven test over every permission-guarded command × {no session, `viewer`, `member`, `admin`, `super_admin`} with expected outcomes from the default matrix; tests for wildcard matching, last-super-role protection and unknown-role rejection; the audit's DevTools bypass (`save_profile` with `role: super_admin` by a member) is reproduced in a core test and now fails with `PermissionDenied`.
  **Verify:** `cargo test -p caf-core capabilities:: -- --nocapture` (paste the matrix).

- [ ] **F8.4 PIN rules and back-off (3.9, D-7).**
  **Do:** PIN 4–8 digits, Argon2id PHC hash in the vault (keep 3.7 behaviour). `select_profile` on a role in `pin_required_for` without a PIN returns `AuthError::PinRequired` (fix `profiles.rs ~191-197`, which returns `true` for no PIN). Per-profile back-off in table `pin_attempts(profile_id PRIMARY KEY, failures INTEGER, next_allowed_at INTEGER)` inside the vault (excluded from the sync guard): 5 free, then 30 s doubling to 15 min; 10 consecutive failures across profiles lock the vault (zeroize key). A profile with `users:manage` can reset another profile's PIN, which forces the member to set a new PIN at next selection.
  **Accept:** fake-clock tests for back-off, vault lock after 10 failures, PIN reset by `users:manage`, member must set PIN after reset, PIN never serialised (`has_pin` only).
  **Verify:** `cargo test -p caf-core profiles::pin`.

- [ ] **F8.5 Auto-lock timers in core (3.10 core part, D-7).**
  **Do:** a core timer task (injected clock) checks idle time: profile idle > `profile_lock_after` → clear session, emit `session-locked`; idle > `vault_lock_after` → lock vault (zeroize), emit `vault-locked`. Activity = any guarded command plus a throttled `heartbeat` command the UI calls on user input (at most every 30 s). Timers are settings stored in `app_kv`, editable with `security:manage`.
  **Accept:** fake-clock tests for both timers, independence (profile lock does not lock the vault), heartbeat resetting idle time, and settings bounds (1–120 min).
  **Verify:** `cargo test -p caf-core session::autolock`.

- [ ] **F8.6 Profile index outside the vault (3.14 core).**
  **Do:** `profile_index.json` (0600) contains only `{ hide_names: bool, profiles: [{ id, display_name, avatar }] }`, rewritten whenever profiles change while unlocked; readable while locked through `profile_index` (Public). When `hide_names` is true, the command returns avatars with generic labels.
  **Accept:** test asserts the JSON key set is exactly the allowed one (no role, PIN, birth year or other field); hide-names test.
  **Verify:** `cargo test -p caf-core profiles::index`.

- [ ] **F8.7 Reset vault rules.**
  **Do:** unlocked: `reset_vault` requires `security:manage` and the typed phrase `RESET <SLUG>` (upper-case slug). Locked (forgotten password, no recovery code): `reset_vault_locked` is `Public`, requires the same phrase plus a second confirmation token returned by a first call, and deletes only the files the vault created (listed constant), never the whole folder.
  **Accept:** tests for both paths, wrong phrase, and that an unrelated file placed in the data dir survives.
  **Verify:** `cargo test -p caf-core vault::reset`.

- [ ] **F8.8 Profiles disabled mode (S6.8).**
  **Do:** when `modules.profiles = false`, setup creates one implicit profile with the super role, and `unlock_vault` selects it automatically (no picker, no PIN).
  **Accept:** core test compiled with a generated config that disables profiles (test-only config injection, not a second build) shows unlock → session exists → `*` permissions.
  **Verify:** `cargo test -p caf-core session::profiles_disabled`.

- [ ] **F8.9 CLI follows the same rules (S-7).**
  **Do:** `caf-cli profile add` prompts for the PIN; CLI commands that touch data require `--profile <id>` and prompt for its PIN, then use the same `guarded` path.
  **Accept:** `caf-cli profile add --help` has no `--pin`; integration test proves a `member` CLI session cannot add a profile.
  **Verify:** `cargo test -p caf-cli`.

**Gate F8:** VS green; RBAC matrix printed; no identity or permission comes from the frontend.

---

### F9 — Visibility and ownership

**Fixes:** C-1 (data isolation part), 3.12, S-7 (`notes list` bypass) · `task.md` S6.6 · D-2 · **Runner:** any · **Depends on:** F8 · **Size:** M

- [ ] **F9.1 Visibility helpers.**
  **Do:** `CORE/visibility.rs`: `enum Visibility { Shared, PrivateSummary, Private }`; `scope(me)` returns the SQL fragment `deleted_at IS NULL AND (owner_profile_id = :me OR visibility = 'shared')` with bound parameters; `can_modify(row, me)` = `row.owner_profile_id == me`, or the session has `<entity>:update`/`delete` on others' rows granted by RBAC. **Super-role exception (D-2 revised):** for the super role `scope` drops the visibility condition (keeps `deleted_at IS NULL`), and every query that returns another profile's `private` rows writes one `PRIVATE_READ` audit event (profile, entity, row count) in the same transaction. No other role bypasses visibility.
  **Accept:** unit tests for the fragment, `can_modify`, and the audit event on super-role reads (one event per query, none when no foreign private row is returned).
  **Verify:** `cargo test -p caf-core visibility::`.

- [ ] **F9.2 Aggregates for `private_summary`.**
  **Do:** `summary(entity, me)` returns per-owner counts (and per-entity numeric aggregates declared by the module) for other profiles' `private_summary` rows, never titles, bodies, tags or ids. For notes: count per owner profile.
  **Accept:** test: B sees A's `private_summary` count but no field of the rows.
  **Verify:** `cargo test -p caf-core visibility::summary`.

- [ ] **F9.3 Apply to every data path.**
  **Do:** `notes.rs` list/get/search/update/delete use `scope`/`can_modify` (remove `… || is_owner` at `~71`; the only bypass is the audited super-role path); get of an invisible row returns `NotFound` (no existence leak); update/delete of a visible row owned by someone else returns `PermissionDenied`. `backup.rs ~26` no longer calls `list_notes("backup-root", true)`; backup reads raw tables under the `backup:export` permission (super role by default). `caf-cli notes list` uses the selected profile (S-7). Any new data command must be added to the visibility test list (F9.4), enforced by an arch test listing commands whose response contains entity rows.
  **Accept:** `git grep -n -E 'backup-root|cli-root' -- crates` empty.
  **Verify:** the grep.

- [ ] **F9.4 Isolation matrix test.**
  **Do:** `crates/caf-core/tests/visibility_matrix.rs`: profiles S (`super_admin`), A (`admin`), M1 and M2 (`member`), V (`viewer`); each creates notes in all three visibilities where permitted; for each data-returning API and each pair, assert what is visible, modifiable and deletable. Include: `admin` A reading M1's `private` (must fail); `super_admin` S reading M1's `private` (succeeds, audit event written, UI flag `foreign_private = true` in the DTO).
  **Accept:** matrix printed in evidence; zero rows of another profile's `private` returned to any non-super role; one audit event per super-role read.
  **Verify:** `cargo test -p caf-core --test visibility_matrix -- --nocapture`.

**Gate F9:** VS green; matrix proves FR-6 isolation for every non-super role and the audited super-role exception.

---

### F10 — Lock-screen UX: two levels, recovery, auto-lock

**Fixes:** T-2 (UI), T-3 (UI), 3.8, 3.10 (UI), 3.14 (UI) · `task.md` S6.7, S6.9 · D-5, D-7 · **Runner:** any + owner smoke · **Depends on:** F9 · **Size:** L

- [ ] **F10.1 Lock screen state machine.**
  **Do:** implement the lock flow as an explicit state machine in `FE/lib/stores/lock.svelte.ts`, driven by `vault_status`: `Legacy` → `Setup` → `ShowRecovery` → `ConfirmRecovery` → `Unlock` ⇄ `RecoveryUnlock` → `SetNewPassword` → `ProfilePicker` → `PinPad` → `App`. `profiles = false` skips picker and PIN pad. `Legacy` shows the D-5 explanation and the data-dir path, with no delete button.
  **Accept:** vitest tests for every transition with mocked API, including back-off countdown from `retry_after_ms`, `PinRequired`, and `RecoveryUnconfirmed`.
  **Verify:** `npm run test -- lock`.

- [ ] **F10.2 Screens.**
  **Do:** setup (password ≥ 12 with strength hint), recovery words display (24 words, numbered, copy and print buttons, "I have saved them"), confirmation of 3 words, unlock (password or "use recovery code"), profile picker from `profile_index` (avatars, names or hidden), PIN pad with on-screen digits ≥ 44 × 44 px and keyboard input, visible focus. All text via `t()` (`lock.*` keys added now).
  **Accept:** `missing_i18n_keys` ratchet shrinks by at least the 37 `lock.*` keys; Playwright screenshots of each screen in both themes against `vite preview` with a mocked Tauri bridge (F14.9 harness may be introduced here) saved under `EVID/F10/`.
  **Verify:** guard output · screenshot file list.

- [ ] **F10.3 Remove the localStorage profile.**
  **Do:** delete the CATerm-style single profile in `FE/lib/stores/profile.svelte.ts ~10`; the active profile comes from `current_session`. Only UI preferences (theme, locale) may use `localStorage`, wrapped in try/catch (`prompt-dev.md` §6).
  **Accept:** `git grep -n localStorage -- frontend/src` lists only theme and locale keys.
  **Verify:** the grep.

- [ ] **F10.4 Auto-lock wiring.**
  **Do:** listen to core events `session-locked` and `vault-locked`; send throttled `heartbeat` on pointer/keyboard activity; on `session-locked` return to the picker, on `vault-locked` to unlock; clear in-memory page data on both.
  **Accept:** vitest test with fake timers and mocked events; OWNER-VERIFY: set timers to 1 min and observe both locks.
  **Verify:** `npm run test -- autolock` · OWNER-VERIFY ID.

- [ ] **F10.5 Profiles & Security settings section.**
  **Do:** list profiles; profiles with `users:*` can add/edit/delete profiles and assign roles, reset a member PIN; a read-only "Roles & Permissions" view shows the current matrix (`roles:view`; editing arrives in v0.2.0 G1); every profile can change its own PIN; toggle "hide names on lock screen"; `security:manage` sets timers, changes the master password and regenerates the recovery code (show-once + confirm); help text stating the shared-device limitation and that the super role can read private data (FR-6).
  **Accept:** wiring chain (§1.2.4) for each action; a profile without the permission sees no control **and** the core rejects the call anyway (F8 test referenced).
  **Verify:** vitest tests per action · evidence links to F8 tests.

- [ ] **F10.6 Profile menu and palette.**
  **Do:** `ProfileMenu` shows the session profile and role; palette command "Switch profile" locks the profile session.
  **Accept:** vitest test.
  **Verify:** `npm run test -- profileMenu`.

- [ ] **F10.7 Security docs (S6.9).**
  **Do:** `docs/security.md` sections: two lock levels, key hierarchy (link to ADR 0001), recovery code handling, back-off and its limit, profile separation is app-level on a shared device, file permissions, zeroization limits, legacy-format policy (D-5).
  **Accept:** file exists; every claim links to the test that proves it.
  **Verify:** `grep -c 'tests/' docs/security.md`.

- [ ] **F10.8 Owner smoke test (S6.7 wiring proof).**
  **Do:** OWNER-VERIFY script: fresh `CAF_DATA_DIR`; `cargo tauri dev`; setup → save words → confirm → create profiles A (super_admin) and B (member, PIN) → restart app → picker shows both → wrong PIN 6 times shows countdown → correct PIN works → recovery unlock path → change password → restart → new password works. Owner pastes screenshots or a screen recording link and the terminal log.
  **Accept:** owner output recorded → stage `DONE`; until then `GATE_PASSED (agent)`.
  **Verify:** OWNER-VERIFY ID.

**Gate F10:** VS green; lock flow tests green; screenshots of every screen in both themes; owner smoke queued or done.

---

### F11 — Backup v2 and scheduled backup

**Fixes:** T-5 · `task.md` S5.7, S5.7b · FR-9 · D-2 · **Runner:** any + owner smoke · **Depends on:** F10 · **Size:** L

Format (record as `docs/adr/0002-backup-format.md`):

```
magic "CAFBAK\0\0" (8 bytes) | format_version u16 = 2 | header_len u32 | header JSON | ciphertext
header: { app_slug, app_code, framework_version, schema_version, created_at, key_mode: "passphrase" | "vault",
          kdf: { alg: "argon2id", m, t, p, salt } (passphrase mode),
          key_slots: { password: {...}, recovery: {...} } (vault mode: copies of the keyring slots wrapping BK),
          cipher: "aes-256-gcm", nonce }
ciphertext = AEAD(file_key, zstd(JSON dump), aad = magic||version||header)
```

- [ ] **F11.1 Export (manual, passphrase mode).**
  **Do:** `export_backup(path, passphrase)` (requires `backup:export`): passphrase ≥ 12 characters; random salt per backup (remove the fixed salt `b"caframework.vault.backup.salt.2026"` at `backup.rs ~47,65`); payload = every sync table **including tombstones**, `change_log`, `profiles` with `pin_hash`, `app_kv` (AI key included, encrypted by the backup key; stated in the UI), `schema_version`. File written with `write_private_atomic()` and atomic rename.
  **Accept:** tests: two exports of the same data have different salts and ciphertexts; header readable without the passphrase and contains no secret; file mode 0600.
  **Verify:** `cargo test -p caf-core backup::export`.

- [ ] **F11.2 Restore (replace mode, all or nothing).**
  **Do:** `import_backup(path, passphrase)` (requires `backup:import`; also allowed during `Setup` on an empty vault, so a new device can restore): verify magic, version (`UnsupportedVersion` for unknown), app slug (`WrongApp`), AEAD (`IntegrityFailed`), `schema_version` ≤ current (run migrations after load when lower); then in **one transaction** delete all rows of the payload tables and insert the payload rows verbatim. Any error rolls back and is returned; no `let _ =` anywhere in `backup.rs`.
  **Accept:** `git grep -n 'let _ =' -- crates/caf-core/src/backup.rs` empty; tests for every error variant; a failure injected after half the inserts leaves the DB unchanged.
  **Verify:** `cargo test -p caf-core backup::import` · the grep.

- [ ] **F11.3 Round-trip test (FR-9).**
  **Do:** `tests/backup_roundtrip.rs`: create 3 profiles (two with PIN), notes in all visibilities including soft-deleted ones, AI settings; export; create a **new** data dir with a different master password; import; compare a canonical dump of every table (sorted by primary key) byte for byte; verify PINs still work and tombstones are preserved. Also a 10 000-note run with its timing recorded.
  **Accept:** test passes; dump hashes before/after pasted; timing recorded.
  **Verify:** `cargo test -p caf-core --test backup_roundtrip -- --nocapture`.

- [ ] **F11.4 Scheduled backup (vault key mode).**
  **Do:** scheduled backups need no passphrase: they encrypt a random file key with BK (F6), and the header carries copies of the keyring's password and recovery slots wrapping BK, so the backup can be restored on a new device with the master password valid at backup time **or** the recovery code. Settings (`backup:manage`): folder (dialog plugin), interval daily/weekly/monthly (default weekly), retention (default 8 files, only files matching `<slug>-backup-YYYYMMDD-HHMMSS.cafbak` are pruned), overdue reminder (default 30 days), visible "last successful backup" time. Runs in the app process while the vault is unlocked; failures notify and do not update the last-success time. Documented limit: no backup while the app is closed.
  **Accept:** fake-clock tests: due/not due; overdue after 30 days triggers a notification event; failed write keeps old last-success; retention keeps 8 and never deletes foreign files; restore of a scheduled backup with password and, separately, with recovery code on a fresh data dir.
  **Verify:** `cargo test -p caf-core backup::schedule -- --nocapture`.

- [ ] **F11.5 UI.**
  **Do:** Backup & Restore settings section: export (passphrase twice), import (file picker, passphrase or password/recovery for vault-mode files, explicit "this replaces all data" confirmation), schedule controls, last success, overdue banner.
  **Accept:** wiring chain per action; `unregistered_invokes` has no backup entries.
  **Verify:** vitest tests · guard output.

- [ ] **F11.6 Owner smoke test.**
  **Do:** OWNER-VERIFY: fresh data dir, create data, export, wipe (new `CAF_DATA_DIR`), import, compare; set schedule to daily with a temp folder and fake-advance not possible in GUI, so check that "Back up now" writes a correctly named file.
  **Accept:** owner output recorded.
  **Verify:** OWNER-VERIFY ID.

**Gate F11:** VS green; round trip identical; every failure mode returns a specific error; ADR committed.

---

### F12 — Module switches, endpoints, Pro, feedback, crash, updater

**Fixes:** 2.4, S-6, S-8, S-9 (capability part), 6.10, T-4 (remaining 11 invokes) · `task.md` S8.1–S8.4, S5.8 · D-4 · **Runner:** any (+ VS-A) · **Depends on:** F11 · **Size:** L

- [ ] **F12.1 Module switches end to end (2.4).**
  **Do:** core: `guarded` returns `AppError::ModuleDisabled { module }` for commands of a disabled module (policy registry gains a `module` field). Frontend: nav items, settings sections, palette commands, the "Ask AI" button (`+layout.svelte ~198-206`), Pro UI (including the LockScreen Pro login mode) and Support page render only when `APP_CONFIG.modules.<m>` is true **and**, for network features, the endpoint flag is set.
  **Accept:** core tests per module; vitest tests render the layout with each module off and assert the element is absent; screenshot pair (all on / all off).
  **Verify:** `cargo test -p caf-core modules::` · `npm run test -- modules`.

- [ ] **F12.2 Endpoints from `app.toml` (S-6).**
  **Do:** remove hard-coded URLs in `CORE/crash.rs ~21`, `CORE/feedback.rs ~10`, `tauri.conf.json ~51`; use `GCC_BASE_URL`/`UPDATER_MANIFEST_URL`; empty = feature disabled at build time. Move website/pricing/sponsor links from `FE/lib/appInfo.ts ~7-15` into a product-editable `appInfo.ts` with empty framework defaults (About hides empty links); record the possible `[links]` section in `prd-amendments.md` instead of adding it.
  **Accept:** `git grep -n -E 'https?://[a-z0-9.-]*fathforce' -- crates frontend/src` empty (only docs and `app.toml` comments may mention it).
  **Verify:** the grep.

- [ ] **F12.3 One HTTP client.**
  **Do:** `CORE/http.rs`: connect timeout 10 s, request timeout 30 s (AI 120 s), user agent `<slug>/<version> (+<brand.website>)`, HTTPS only except loopback, header `X-CA-App-Id: <identifier>` and JSON field `app_id` on every GCC call (FR-8); errors map to `NetError::{Timeout, Offline, Http{status}, Tls}`.
  **Accept:** tests against a local mock server (`httpmock` or `wiremock`) assert the header, the field, and the timeout error.
  **Verify:** `cargo test -p caf-core http::`.

- [ ] **F12.4 Pro licensing client (D-4).**
  **Do:** implement the 10 licensing commands in core per `docs/backend-contract.md` (written now: endpoints, request/response JSON, headers, error codes, `app_id` rule); session token stored in the vault (`app_kv`), never in `localStorage`; license signature public key taken from the build env `<CODE>_PRO_LICENSE_PUBKEY_V1`, Pro disabled when absent; delete `ProTeamPanel`, the team/sync API functions and their strings.
  **Accept:** mock-server tests for each command (success and typical failure); `unregistered_invokes` loses the 10 `pro_*` entries; `git grep -n -E 'pro_team|pro_sync|ProTeamPanel' -- crates frontend/src` empty; with `pro = false` (framework) no Pro UI renders.
  **Verify:** `cargo test -p caf-core pro::` · guard output · the grep.

- [ ] **F12.5 Feedback via GCC proxy.**
  **Do:** `submit_feedback` with `app_id`, disabled when no endpoint; no secret in the client.
  **Accept:** mock-server test; real request **NOT VERIFIED** unless an endpoint exists (state why).
  **Verify:** `cargo test -p caf-core feedback::`.

- [ ] **F12.6 Shared PII scrubber (S-8, 6.10).**
  **Do:** `CORE/pii.rs` used by crash reports and AI: emails (fix the `[A-Z|a-z]` class), payment cards 13–19 digits with Luhn check and common separators, Indonesian mobile numbers (`+62`/`62`/`08`, 9–13 digits), NIK (16 digits with a plausible province/date structure), NPWP (15/16 digits, dotted or plain), IBAN, bearer tokens and `sk-`/`gsk_` keys, IPv4, home paths and usernames (existing crash rules kept). Each rule has positive and negative tests (for example a 16-digit non-Luhn order number is **not** redacted as a card).
  **Accept:** ≥ 30 scrubber tests; crash tests show email and card redacted in panic messages.
  **Verify:** `cargo test -p caf-core pii:: crash::`.

- [ ] **F12.7 Crash reports end to end (S5.8).**
  **Do:** register `submit_crash_report` (opt-in preview, scrubbed payload, `app_id`, endpoint required); `dismiss_crash_report(id, never_again)` persists `never_again`; panic hook stays first; dumps stay `0600`.
  **Accept:** `unregistered_invokes` ratchet is empty (0 lines); test: after `never_again`, the next start does not offer the report; forced-panic test writes a scrubbed dump.
  **Verify:** guard output · `cargo test -p caf-core crash::`.

- [ ] **F12.8 Updater configuration and Android capability (S-9).**
  **Do:** updater pubkey and endpoint come from codegen; remove `updater:default` from `capabilities/default.json ~37` and keep it only in `desktop.json`; local test manifest in `crates/caf-app/tests/fixtures/latest.json` for an update-check test.
  **Accept:** VS-A: update check against the local manifest reports "update available" for a higher version; `grep -n updater crates/caf-app/capabilities/default.json` empty.
  **Verify:** `cargo test -p caf-app updater` (VS-A) · the grep.

**Gate F12:** VS green; ratchet `unregistered_invokes` deleted or empty; every module can be switched off without dead UI; no hard-coded private endpoint.

---

### F13 — AI assistant, FR-7 complete

**Fixes:** T-10, 6.2, 6.4, 6.5, 6.6, 6.7, 6.8 · `task.md` S7.1–S7.5 · D-2 · **Runner:** any + owner check · **Depends on:** F12 · **Size:** L

- [ ] **F13.1 Modes and providers (6.2).**
  **Do:** `AiMode::{Off, Byo, Hosted}`; `Hosted` only when `modules.pro` and a GCC endpoint exist, otherwise rejected and hidden. BYO uses one OpenAI-compatible client with an explicit `base_url` (no default to `api.openai.com`; the UI offers presets for OpenAI, Ollama `http://localhost:11434/v1`, LM Studio). Remove the `anthropic`/`hosted` fall-through at `ai.rs ~147-154`; unknown providers return `AiError::UnsupportedProvider`. Changing `base_url` to a different host clears the stored key unless the user re-enters it.
  **Accept:** mock-server test: the `Authorization` header is sent only to the configured host; switching host clears the key; `Off` makes `ai_chat` return `AiError::Disabled`.
  **Verify:** `cargo test -p caf-core ai::mode`.

- [ ] **F13.2 Configurable fields (6.7).**
  **Do:** `base_url`, `model`, `temperature` (0.0–2.0, sent to every provider including Ollama), `response_language` (`auto`/`id`/`en`), `tone` (`neutral`/`friendly`/`formal`/`concise`), `custom_instructions` (≤ 2000 characters), `privacy_level_default`, `allow_full_detail_for_local`. `get_ai_settings` returns `has_api_key: bool`, never the key; `set_ai_api_key`/`clear_ai_api_key` are separate commands requiring `ai:manage`.
  **Accept:** tests for bounds and that no response type contains the key (serialize all IPC types in a test and grep for the test key).
  **Verify:** `cargo test -p caf-core ai::settings`.

- [ ] **F13.3 Context providers and privacy levels (6.5).**
  **Do:** trait `ContextProvider { fn id(&self); fn build(&self, core, session, level) -> ContextPayload }` with a registry products extend in one file; framework providers `app` (profile count, enabled modules) and `notes` (sample). Levels: `Summary` (default; aggregates only, no names, no free text), `Detailed` (requires per-conversation consent, held in memory and cleared on lock), `Full` (only when `allow_full_detail_for_local` and `base_url` host is loopback). Providers read through the visibility scope (D-2). Command `ai_preview_context(level)` returns the exact payload for the consent dialog.
  **Accept:** payload tests per level using fixture profiles named "Siti Aminah" and "Budi Santoso" and notes containing emails and card numbers: `Summary` contains neither name nor any note text; `Detailed` without consent returns `ConsentRequired`; `Full` refused for a non-loopback host; other profiles' `private` notes never appear at any level.
  **Verify:** `cargo test -p caf-core ai::context -- --nocapture` (paste the summary payload).

- [ ] **F13.4 Guardrails (6.6).**
  **Do:** product guardrail text lives in `crates/caf-core/src/ai/guardrails.txt` (compiled in; generator keeps it per product); composition order: guardrail → framework format rules → user custom instructions → context → user message. Remove the "Enforce System Guardrails" toggle and any setting that can disable guardrails.
  **Accept:** test asserts the composed message order and that no setting changes the guardrail section; `git grep -n -i 'guardrails_enabled\|Enforce System Guardrails' -- crates frontend/src` empty.
  **Verify:** `cargo test -p caf-core ai::compose` · the grep.

- [ ] **F13.5 Scrubbing (6.4).**
  **Do:** apply `pii.rs` to the user prompt **and** the context before sending; return `redactions_applied`.
  **Accept:** mock-server test inspects the outgoing body: no email, card, phone, NIK or NPWP from the fixture prompt.
  **Verify:** `cargo test -p caf-core ai::scrub`.

- [ ] **F13.6 Errors (6.8).**
  **Do:** remove the fake success at `ai.rs ~129-138`; return `AiError::NotConfigured` (expected `CAF-AI-002`), `QuotaExceeded` (HTTP 402/429 in Hosted), `Network`/`Timeout`, `ProviderError { status }`, `ConsentRequired`, `Disabled`. Hosted errors are shown to the user; BYO errors are shown inline in the panel without modal interruption (interpretation of "fall back quietly", recorded in `prd-amendments.md` for the owner to confirm).
  **Accept:** one test per error; frontend test renders each error state via `errorText()`.
  **Verify:** `cargo test -p caf-core ai::errors` · `npm run test -- ai`.

- [ ] **F13.7 UI.**
  **Do:** `AiChatPanel`: renders `.message`, redaction badge, privacy level selector, consent dialog with payload preview, four states; AI settings section with all F13.2 fields, presets and key status.
  **Accept:** wiring chain per action; screenshots both themes.
  **Verify:** vitest tests · screenshot files.

- [ ] **F13.8 Real completion (owner check).**
  **Do:** OWNER-VERIFY: `ollama serve` + `ollama pull llama3.2`, BYO preset Ollama, send a message; paste the visible answer and the app log line showing `redactions_applied`.
  **Accept:** owner output recorded.
  **Verify:** OWNER-VERIFY ID.

**Gate F13:** VS green; payload tests prove the privacy levels; no key leaves the core.

---

### F14 — Shell and design system

**Fixes:** T-8 (shell, theme, accent, layout), 5.4–5.10, R-4 · `task.md` S4.1–S4.7 · `prompt-dev.md` §4 · **Runner:** any + owner visual · **Depends on:** F13 · **Size:** L

- [ ] **F14.1 Title bar (5.8).**
  **Do:** `TitleBar.svelte` in the shell after unlock (reuse the LockScreen implementation): `data-tauri-drag-region`, minimise, maximise/restore, close, double-click to maximise; hidden on Android by a build-time constant from codegen (`APP_PLATFORM`) **and** a runtime check (`@tauri-apps/plugin-os` or equivalent). Window permissions for these actions present in the desktop capability.
  **Accept:** vitest test for visibility logic; OWNER-VERIFY on x1-bench: drag, minimise, maximise, close work.
  **Verify:** `npm run test -- titlebar` · OWNER-VERIFY ID.

- [ ] **F14.2 Sidebar, groups, role filter, drawer (5.9).**
  **Do:** sidebar from `navItems.generated.ts`, grouped by `group` (labels via i18n), filtered by session role (no-op when profiles are off) and module switches, 24 × 24 outline icons; below `md` a drawer `w-72 max-w-[85vw]` with backdrop, focus trap, Esc to close.
  **Accept:** unit test of the role/group filter; Playwright screenshots at 1280 px and 390 px.
  **Verify:** `npm run test -- nav` · screenshot files.

- [ ] **F14.3 Tokens and dark/light everywhere (5.4).**
  **Do:** keep `app.css` tokens identical to `prompt-dev.md` §4 (diff only intentional changes); replace hard-coded dark classes in shell, Notes, Settings, `PageHeader`, state components, `AiChatPanel`, `CommandPalette` with token utilities or `dark:` pairs; theme store toggles `.dark` on `<html>` (light/dark/system).
  **Accept:** `raw_color_classes` ratchet shrinks to state colours only; Playwright screenshots of every page in both themes.
  **Verify:** guard output · screenshots.

- [ ] **F14.4 Accent map (5.5).**
  **Do:** `FE/lib/accent.ts` exports a literal class map for the 10 accents (text, bg, soft bg, border, ring), each value a full literal string; `APP_CONFIG.accent` and menu overrides select from it; remove all 62 hard-coded `indigo-*` uses.
  **Accept:** `raw_color_classes` ratchet empty; a production build test greps the built CSS for each accent's classes (proves no class was purged); screenshot with two different accents in a temp `app.toml`.
  **Verify:** `grep -c 'emerald' frontend/build/_app/immutable/assets/*.css` (non-zero) · guard output.

- [ ] **F14.5 `PageHeader` and `PageContainer` on every page (5.7).**
  **Do:** `PageHeader` props: icon path, accent, title, badge/subtitle, actions slot (Svelte 5 snippet); every route uses `PageContainer` with the exact classes from `prompt-dev.md` §4.
  **Accept:** arch-style vitest/script test: every `+page.svelte` under `FE/routes` (except `_dev`) imports and renders `PageContainer` and `PageHeader`.
  **Verify:** `npm run test -- pages`.

- [ ] **F14.6 Four states and backend-unavailable (5.6).**
  **Do:** add `BackendUnavailable.svelte` (shown when not running in Tauri or when IPC transport fails); every data page renders loading, empty, error (`ErrorState` with `errorText`) and backend-unavailable; Notes no longer shows `EmptyState` on error. Dev-only route `/_dev/states` shows all four in both themes and is excluded from production builds.
  **Accept:** Playwright screenshots of `/_dev/states`; `ls frontend/build | grep -c _dev` = 0 after `npm run build`.
  **Verify:** screenshots · the command.

- [ ] **F14.7 Settings registry (5.10, FR-3).**
  **Do:** `FE/lib/settings/registry.ts` with `registerSettingsSection({ id, labelKey, icon, order, component, requires?: ModuleKey, capability? })`; framework sections Appearance, Language, Profiles & Security, Backup & Restore, AI Assistant, Updates, Privacy (crash reports), Shortcuts, Pro (if enabled), About; the settings page only iterates the registry.
  **Accept:** test registers a section from a fixture file outside `settings/+page.svelte` and asserts it renders, with `git diff --stat` showing the page unchanged.
  **Verify:** `npm run test -- settings-registry`.

- [ ] **F14.8 Palette, shortcuts, Logo (R-4).**
  **Do:** palette lists visible menus and global commands from one `shortcuts.ts`; `Logo.svelte` uses `$props()`.
  **Accept:** `git grep -n 'export let' -- frontend/src` empty; palette test.
  **Verify:** the grep · `npm run test -- palette`.

- [ ] **F14.9 Automated visual harness and accessibility.**
  **Do:** Playwright (Chromium is pre-installed on most runners) against `vite preview` with a mocked `window.__TAURI_INTERNALS__`, producing screenshots of: lock screens, shell, Notes, Settings sections, `/_dev/states`, at 1280 × 800 and 390 × 844, light and dark. Accessibility checks: `svelte-check` a11y warnings = 0; `@axe-core/playwright` with no serious/critical violations; contrast test computing WCAG ratios for `--text-primary`/`--text-secondary` on `--bg-app`/`--bg-surface` and every accent text class on surfaces, both themes (AA).
  **Accept:** screenshot set in `EVID/F14/`; axe report attached; contrast table pasted.
  **Verify:** `npx playwright test` · contrast test output.

- [ ] **F14.10 Owner visual comparison with CATerm.**
  **Do:** OWNER-VERIFY: owner compares the F14.9 screenshots with CATerm (desktop and narrow, dark and light) and answers "structurally identical: yes/no + notes" (PRD success criterion 2).
  **Accept:** owner answer recorded.
  **Verify:** OWNER-VERIFY ID.

- [ ] **F14.11 Brand surfaces (D-11, FR-18).**
  **Do:** About shows the full brand block (developer, company, website, email, phone formatted per locale, support URL, copyright with the current year, logo, app version, `caframework_version`); lock screen shows a small footer (developer · website · email) when `show_on_lock_screen`; links open through `open_external_url`; all from `generated/brand.ts`, never hard-coded.
  **Accept:** vitest tests render About and the lock footer from a fixture brand; `git grep -n -E 'cecepazhar|hi@cecepazhar' -- frontend/src crates` hits only generated files; screenshots both themes.
  **Verify:** `npm run test -- brand` · the grep.

**Gate F14:** VS green; `raw_color_classes` empty; screenshots and a11y reports attached.

---

### F15 — i18n completion and formatting

**Fixes:** 5.3, 5.11, T-8 (raw keys, hard-coded strings) · `task.md` S4.8, S2.5 · FR-10 · **Runner:** any · **Depends on:** F14 · **Size:** M

- [ ] **F15.1 Type-safe `t()`.**
  **Do:** `type Key = Paths<Dictionary>`; `t(key: Key, params?)`; a separate `tDynamic(key: string)` for runtime keys (for example generated menu labels) that logs a dev warning and returns a readable fallback, never the raw key.
  **Accept:** a deliberately wrong literal key in a test file makes `npm run check` fail (show output, then remove the file).
  **Verify:** `npm run check` before/after.

- [ ] **F15.2 Complete the dictionaries.**
  **Do:** add every missing key (baseline 134: lock 37, pro 25, feedback 22, crash 13, about 12, profileMenu 10, updater 6, others 9, minus keys deleted with removed features) to `en.ts` and `id.ts`; Indonesian written as natural Indonesian, not literal translation.
  **Accept:** `missing_i18n_keys` ratchet empty; vitest test: no empty value, every `id` key exists, and a listed allow-set is the only place where `id` equals `en`.
  **Verify:** guard output · `npm run test -- i18n`.

- [ ] **F15.3 Move hard-coded strings.**
  **Do:** move user-visible English text (Settings labels, toasts, "Ask AI", "Lock Vault", defaults in `EmptyState`/`LoadingState`/`ErrorState`, aria-labels) into the dictionaries.
  **Accept:** `hardcoded_strings` ratchet empty.
  **Verify:** guard output.

- [ ] **F15.4 Formatting helpers (5.11).**
  **Do:** `FE/lib/format.ts`: `formatNumber`, `formatCurrency` (IDR without decimals by default, USD with 2), `formatDate`, `formatDateTime`, `formatRelative`, all `Intl`-based and following the active locale; replace `formatUsd` (`pro/pricing.ts ~22`).
  **Accept:** tests for `id` and `en`, for example `formatNumber(1234567.5)` → `1.234.567,5` / `1,234,567.5`, IDR and USD outputs (normalise NBSP in assertions), dates in both locales.
  **Verify:** `npm run test -- format`.

**Gate F15:** VS green; both i18n ratchets empty; formatting tests green.

---

### F16 — Notes sample completion

**Fixes:** FR-11 (visibility selector, import/export JSON, real profiles), success criterion 3 · `task.md` S9.1–S9.5 · **Runner:** any + owner smoke · **Depends on:** F15 · **Size:** M

- [ ] **F16.1 Core and migration complete.**
  **Do:** notes migration in the sample range (`0500–0599`): `title`, `body`, `tags` (JSON array of lower-case strings, max 20), sync columns, owner and visibility from the session; validation errors are specific (`NotesError::{TitleRequired, TitleTooLong, TooManyTags, …}`).
  **Accept:** core tests for validation and CRUD through `sync.rs` and `visibility.rs`.
  **Verify:** `cargo test -p caf-core notes::`.

- [ ] **F16.2 Import/export JSON.**
  **Do:** export the session profile's own notes plus visible shared notes as JSON (schema version field) via the dialog/fs plugins; import assigns the session profile as owner, keeps ids when not present, and reports created/updated/skipped counts.
  **Accept:** core round-trip test; vitest test of the page action.
  **Verify:** `cargo test -p caf-core notes::json` · `npm run test -- notes`.

- [ ] **F16.3 Page.**
  **Do:** `PageHeader` + `PageContainer`, list/search/tag filter, editor, visibility selector (`shared` / `private_summary` / `private` with one-line explanations), four states, `private_summary` aggregates of other profiles shown as counts only; all text i18n.
  **Accept:** screenshots both themes; wiring chain for create, edit, delete, import, export.
  **Verify:** screenshot files · vitest tests.

- [ ] **F16.4 Sample markers for the generator.**
  **Do:** every Notes-only file is listed in `template/sample-manifest.toml`; shared files mark Notes-only regions with `// @sample-begin notes` … `// @sample-end` (or the HTML/TOML comment equivalent), including the nav entry, i18n keys, migration, commands, policy entries, AI context provider and tests.
  **Accept:** xtask test: stripping the sample from a temp copy leaves a tree that passes `cargo check -p caf-core` and `npm run check` and contains no `list_notes`, `NotesPage`, `notes::` or `notes.` i18n key.
  **Verify:** `cargo test -p caf-xtask --test sample_strip`.

- [ ] **F16.5 Wiring proof (S9.4, success criterion 3).**
  **Do:** automated core scenario test: A creates a private note → B lists → not visible → reopen the DB from disk (simulated restart) → A lists → visible. Plus OWNER-VERIFY on x1-bench with the GUI: same scenario with a real app restart, screenshots.
  **Accept:** test output; owner output recorded.
  **Verify:** `cargo test -p caf-core --test notes_wiring -- --nocapture` · OWNER-VERIFY ID.

- [ ] **F16.6 `docs/adding-a-menu.md`.**
  **Do:** a 14-step checklist from this slice: the 13 steps of `prompt-dev.md` §6 plus "mark sample regions / update the generator manifest if the menu is optional".
  **Accept:** file exists; each step links to the Notes file that shows it.
  **Verify:** link check in F17.

**Gate F16:** VS green; wiring proof test green; owner smoke queued or done.

---

### F17 — Docs and open-source hygiene

**Fixes:** FR-14, S-5 (docs part), D-6 preparation · `task.md` S12.3, S12.4, S13.1, S13.2 · **Runner:** any · **Depends on:** F16 · **Size:** M

- [ ] **F17.1 Repository files.**
  **Do:** `LICENSE` (MIT, owner name), `NOTICE` (CATerm attribution), `README.md` (what it is, quick start, generator, security model summary, status, author block from `[brand]`), `CONTRIBUTING.md`, `SECURITY.md` (reporting contact = `[brand].email`), `CHANGELOG.md` with a "Port notes" section per entry.
  **Accept:** files exist; `README.md` commands copy-pasted and run in a temp clone (output attached).
  **Verify:** `ls LICENSE NOTICE README.md CONTRIBUTING.md SECURITY.md CHANGELOG.md`.

- [ ] **F17.2 Documentation set.**
  **Do:** `docs/architecture.md` (layers, modules, data conventions, lock levels, AI privacy levels), `docs/security.md` (from F10.7), `docs/backend-contract.md` (from F12.4), `docs/error-codes.md` (generated), `docs/android.md` (stub, finished in F19), `docs/creating-an-app.md` (stub, finished in F18), `docs/adding-a-menu.md` (F16), `docs/porting-framework-fixes.md`.
  **Accept:** link checker (offline mode, relative links and anchors) reports 0 broken links.
  **Verify:** `npx markdown-link-check -q docs/*.md README.md` (or `lychee --offline`).

- [ ] **F17.3 Secret scan of the whole history (S12.4).**
  **Do:** `gitleaks detect --source . --log-opts=--all -v`; triage every finding in `F17.md` (true positive → OWNER-VERIFY with rotation advice, false positive → `.gitleaksignore` entry with reason).
  **Accept:** 0 untriaged findings.
  **Verify:** gitleaks output.

- [ ] **F17.4 Publication plan and dry run (D-6).**
  **Do:** write `Notes/publication-plan.md` with exact commands for the owner: (1) create private repo `caframework-notes` and push `Notes/` history into it (`git filter-repo --path Notes/ --path-rename Notes/:` on a separate clone), (2) on another fresh clone remove `Notes/` and `build/bin/caterm` from all history (`git filter-repo --invert-paths --path Notes/ --path build/bin/caterm`), (3) verify, (4) force-push to the public remote. Run steps 1–3 in a temporary clone under the scratch directory and record the repository size before and after (`git count-objects -vH`), the absence check (`git log --all -- build/bin/caterm` empty) and the file list. Never touch the real remote.
  **Accept:** plan file and dry-run output exist; OWNER-VERIFY entry marked `BLOCKING for publication`.
  **Verify:** dry-run logs.

**Gate F17:** VS green; link check clean; gitleaks triaged; publication plan ready for the owner.

---

### F18 — Generator `new-app`

**Fixes:** T-6 (generator), 2.7–2.13, 7.6 (launcher, D-3), success criterion 1 · `task.md` S10.1–S10.7, S12.2 · **Runner:** any (+ x1-bench for VS-A in generated output) · **Depends on:** F17 · **Size:** L

- [ ] **F18.1 Copy from an allow-list (2.12).**
  **Do:** source file list = `git ls-files -z` of the framework (refuse to run on a dirty tree unless `--allow-dirty`, which is recorded in the output's `GENERATED.md`); exclude `Notes/`, `guards/ratchet/` content (empty ratchet files are created fresh), `template/sample-manifest.toml` itself, and anything under `target/`, `node_modules/`, `dist/`, `gen/`, `build/`. `.github/`, `.gitignore`, `.gitattributes` are kept. Refuse an `--out` inside the source repo.
  **Accept:** test on a fixture repo: `.github/` present, `dist/` absent even when present on disk; output size of a real run recorded (target < 20 MB before `npm ci`).
  **Verify:** `cargo test -p caf-xtask --test copy` · `du -sh <out>`.

- [ ] **F18.2 Explicit replacement table (2.7).**
  **Do:** `template/replacements.toml`, applied in declared order (longest match first, word-boundary aware, case-specific rules): identifier `com.fathforce.caframework` → `[app].identifier` (applied **before** the bare `caframework` rule); `CAFramework` → `name`; `caframework` → `slug`; crate names `caf-core|caf-app|caf-cli|caf-xtask` → `<slug>-…` and their Rust paths `caf_core` → `<slug_underscored>_core`; package `caf-frontend` → `<slug>-frontend`; storage keys `caf-theme`, `caf_locale`, `caf_profile_v1` → `<slug>-theme`, `<slug>_locale`, `<slug>_profile_v1`; env vars `CAF_DATA_DIR`, `CAF_PRO_LICENSE_PUBKEY_V1`, `CAFRAMEWORK_COLD_START_MS` → `<CODE>_…`; CLI binary `caframeworkctl` → `<slug>ctl`; CI workflow names. The error prefix is not text-replaced; it comes from codegen. Text files are detected by content (NUL sniff), so `.js`, `.css`, `.html`, `.yml`, `.sh`, `.desktop`, `.xml` and `Cargo.lock` are processed; binaries are copied byte for byte. Directory and file names are renamed by the same table.
  **Accept:** unit tests per rule on a fixture tree, including the ordering bug from T-6 (identifier must come out as configured, not `com.fathforce.<slug>`); a residue check (F18.6) passes.
  **Verify:** `cargo test -p caf-xtask --test replace`.

- [ ] **F18.3 Generated product content (2.11).**
  **Do:** write the product `app.toml` (with `caframework_version` = framework crate version), run codegen inside the output, create one placeholder route per menu (`/m/<key>/+page.svelte` with `PageHeader`, `PageContainer`, `EmptyState` and the comment `UI-ONLY placeholder`), i18n menu keys, Settings entries for enabled modules, permission catalogue, brand files, installer metadata, `Notes/prd.md` and `Notes/task.md` stubs, fresh empty ratchet files, `GENERATED.md` (source commit, options, date), and `packaging/linux/<slug>.desktop` + icon (D-3).
  **Accept:** tree listing of a generated `demoapp` with 2 menus; `desktop-file-validate` passes on the `.desktop` file (install `desktop-file-utils` or record NOT VERIFIED + owner command).
  **Verify:** `find <out> -path '*/m/*' -name '+page.svelte'` · `desktop-file-validate <out>/packaging/linux/demoapp.desktop`.

- [ ] **F18.4 `--with-sample` (2.8).**
  **Do:** with the flag, keep Notes; without it, drop manifest files and strip marked regions (F16.4 logic).
  **Accept:** grep on the output without the flag finds no `list_notes`, `NotesPage`, `notes::`, Notes migration or `notes.` i18n key; with the flag all are present; both outputs pass VS-R and VS-F.
  **Verify:** the greps · VS logs inside each output.

- [ ] **F18.5 New minisign key pair (2.10).**
  **Do:** generate a key pair with the `minisign` crate; write the secret key to `--keys-dir` (default `~/.config/<slug>/updater/`, refuse a path inside `--out` or the source repo), mode 0600, password-protected via a TTY prompt or `--key-password-stdin`; `--unencrypted-key` exists only for tests and prints a warning. Write only the public key into the generated `tauri.conf.json`.
  **Accept:** test: output `pubkey` differs from the framework's and from a second generation; no file under `--out` contains secret-key material (scan for the minisign secret key header).
  **Verify:** `cargo test -p caf-xtask --test keys`.

- [ ] **F18.6 Safety, dry run and residue check (2.9).**
  **Do:** refuse a non-empty `--out`; `--dry-run` prints the plan (file count, excluded paths, every rename, replacement counts per rule, generated files, key path) and creates nothing; after a real run, a built-in residue check fails the command if `caframework|CAFramework|caf-|caf_` remains outside `NOTICE`, `CHANGELOG.md` port notes, `GENERATED.md` and the `caframework_version` field, or if `[brand]` is missing (D-11).
  **Accept:** tests for non-empty refusal, dry-run creating nothing (directory does not exist afterwards), residue detection on a planted string.
  **Verify:** `cargo test -p caf-xtask --test safety`.

- [ ] **F18.7 E2E test (2.13, success criterion 1).**
  **Do:** `crates/caf-xtask/tests/new_app_e2e.rs` (`#[ignore]`, run by VS-E): generate `demoapp` (2 menus, one restricted by role) with and without `--with-sample`, and a third run from the PRD FR-1 example `app.toml`; in each output run VS-R, VS-F and VS-G (and VS-A on x1-bench); generation is deterministic apart from the key pair (compare trees with keys excluded). Add a CI job `e2e` on the work branch (nightly or on demand if too slow for every push).
  **Accept:** full E2E log; CI job defined.
  **Verify:** `cargo test -p caf-xtask --test new_app_e2e -- --ignored --nocapture`.

- [ ] **F18.8 `docs/creating-an-app.md`.**
  **Do:** step-by-step from writing `app.toml` to first launch, including keys, icons, launcher, Android pointer, and what is UI-ONLY in placeholders.
  **Accept:** OWNER-VERIFY: owner follows the doc alone on x1-bench and confirms (S13 gate).
  **Verify:** OWNER-VERIFY ID.

**Gate F18:** VS green; VS-E green; PRD example generates a passing app with no manual edits.

---

### F19 — Android

**Fixes:** FR-13, S-9 · `task.md` S11.1–S11.4 · **Runner:** x1-bench · **Depends on:** F18 · **Size:** M

- [ ] **F19.1 Fresh Android project.**
  **Do:** after the owner deleted the CATerm `gen/android` (F2.6), run `cargo tauri android init` so the package matches `[app].identifier`; confirm no `caterm` path remains.
  **Accept:** `grep -rIl -i caterm crates/caf-app/gen/android` empty; package path `com/fathforce/caframework`.
  **Verify:** the grep · `find crates/caf-app/gen/android -path '*com/fathforce*' -maxdepth 8 -type d`.

- [ ] **F19.2 Build and sign.**
  **Do:** debug and release APK builds using the CATerm Docker/SDK approach; release signing via a keystore **outside the repo** referenced by `keystore.properties` (gitignored) or environment variables; document in `docs/android.md`.
  **Accept:** build logs; APK paths; `sha256sum`; `apksigner verify --print-certs` output; `git status` shows no keystore.
  **Verify:** the commands.

- [ ] **F19.3 Mobile layout checks.**
  **Do:** title bar hidden; drawer navigation; Playwright run at 390 × 844 that measures every interactive element's bounding box and fails under 44 × 44 px (FR-13).
  **Accept:** touch-target report with 0 violations.
  **Verify:** `npx playwright test mobile`.

- [ ] **F19.4 On-device check (S11.3).**
  **Do:** OWNER-VERIFY: install the APK on a real device; check lock screen, setup with recovery words, profile picker, PIN pad, drawer, Notes create/restart/read-back, backup export to a user-chosen location; screenshots.
  **Accept:** owner output recorded.
  **Verify:** OWNER-VERIFY ID.

- [ ] **F19.5 Plugin gaps (S11.4).**
  **Do:** `docs/android.md` lists gaps and fallbacks: updater not on mobile (store/sideload updates), backup folder selection through the Android document picker, file permissions semantics, auto-lock when the app is backgrounded (lock on `pause` event), title bar.
  **Accept:** each gap has a tested or documented fallback.
  **Verify:** file content.

**Gate F19:** APK installs and passes F19.4 on a real device (owner).

---

### F20 — Release readiness, QA re-audit and Definition of Done

**Fixes:** 7.1 (final), 7.5, 7.6, 7.7, NFR performance, PRD §9 · `task.md` S14, S15 · D-6 · **Runner:** x1-bench · **Depends on:** F19 · **Size:** M

- [ ] **F20.1 Clean-clone verification.**
  **Do:** fresh clone of the work branch into a temp dir; run VS-R, VS-F, VS-G, VS-A, VS-E.
  **Accept:** all green; logs attached.
  **Verify:** the logs.

- [ ] **F20.2 Release build, launch and launcher (7.5, 7.6).**
  **Do:** `cargo tauri build`; launch the release binary with a fresh `CAF_DATA_DIR`; install the generated `.desktop` file to `~/.local/share/applications` (owner) and validate.
  **Accept:** OWNER-VERIFY output: app starts, launcher entry appears, `desktop-file-validate` clean.
  **Verify:** OWNER-VERIFY ID.

- [ ] **F20.3 Cold start (NFR performance).**
  **Do:** on the X1 Yoga Gen 3, release build of the empty **generated** app; 10 cold starts after `sync; echo 3 | sudo tee /proc/sys/vm/drop_caches` (owner runs the privileged step) reading `<CODE>_COLD_START_MS`; report median and p90.
  Also run the F5.8 bench suite and the reference dataset checks on the same laptop: list page ≤ 100 ms, search ≤ 300 ms, backup of 100 000 notes ≤ 60 s, idle memory ≤ 250 MB (`/proc/<pid>/status` VmRSS after 5 min idle).
  **Accept:** median ≤ 2000 ms (measured, not estimated) and every budget met; if not, a finding with a profile.
  **Verify:** owner log.

- [ ] **F20.4 Independent read-only QA re-audit (S14).**
  **Do:** owner starts a **separate** session with `prompt-fix.md` Appendix B; the auditor writes `Notes/audit/<date>-reaudit.md` without changing code. Each finding becomes a task `F20.R<n>` (one commit per finding with a regression test), then re-audit, until 0 critical, 0 high and 0 UI-ONLY.
  **Accept:** final re-audit file with zero critical/high and zero UI-ONLY; list of `F20.R*` commits.
  **Verify:** the audit file.

- [ ] **F20.5 Original `task.md` closure.**
  **Do:** fill `Notes/task.md`: tick each S0–S15 box only with a link to evidence (from `Notes/evidence/fix/` via the §7 mapping); mark items that remain `[~]` or `[!]` with reasons.
  **Accept:** every ticked box has a link; counts of `[x]`/`[~]`/`[!]` reported.
  **Verify:** `grep -c '\[x\]' Notes/task.md` etc.

- [ ] **F20.6 Version and changelog.**
  **Do:** set version `0.1.0` in `Cargo.toml` workspace, `package.json`, `tauri.conf.json`; `CHANGELOG.md` entry with port notes; prepare (do not create) the tag command.
  **Accept:** grep shows `0.1.0` in all three; no `0.1.0-dev` left.
  **Verify:** `git grep -n '0.1.0' -- Cargo.toml frontend/package.json crates/caf-app/tauri.conf.json`.

- [ ] **F20.7 CACash skeleton (S15.2).**
  **Do:** generate from the owner's CACash `app.toml` into a separate directory (not a repo the agent pushes); if the owner has not supplied it, use the PRD FR-1 example and label the result "stand-in". Run VS inside it; OWNER-VERIFY for Linux launch and Android APK.
  **Accept:** logs; owner launch confirmations; DoD item open if stand-in.
  **Verify:** logs · OWNER-VERIFY IDs.

- [ ] **F20.8 DoD checklist (PRD §9).**
  **Do:** in `F20.md`, one row per DoD condition and per PRD success criterion with evidence links: all stages passed, clean re-audit, `v0.1.0` tag (owner), CACash skeleton on Linux and Android.
  **Accept:** table complete; open items listed honestly.
  **Verify:** the table.

- [ ] **F20.9 Owner actions (Tier 2).**
  **Do:** hand over: merge the work branch, create tag `v0.1.0`, execute `Notes/publication-plan.md` (D-6), create the release secrets, publish. The agent only prepares the commands.
  **Accept:** owner confirms each action; the agent records the tag hash and publication date.
  **Verify:** `git tag -l v0.1.0` (after owner action).

**Gate F20:** DoD table complete with evidence; owner actions recorded.

---

## 5. Traceability matrix (audit → tasks)

Every audit item and finding maps to at least one task. PASS items map to the task that keeps them passing.

### 5.1 Audit items (73)

| Item | Audit status | Task(s) |
|---|---|---|
| 1.1 | PASS | F0.3 (re-check), F2.3 |
| 1.2 | PASS | F0.3, F2.3 |
| 1.3 | PASS | F0.3, F1.3 |
| 1.4 | PASS | F0.3, F2.2 |
| 1.5 | FAIL | F2.1, F2.2, F2.3, F2.4 |
| 1.6 | PARTIAL | F2.2, F2.3 |
| 1.7 | FAIL | F2.3, F2.4 |
| 2.1 | FAIL | F3.1 |
| 2.2 | PASS | F3.1 (contract replaces it, D-1) |
| 2.3 | FAIL | F3.2 |
| 2.4 | FAIL | F3.1, F12.1 |
| 2.5 | PASS | F3.4 |
| 2.6 | FAIL | F3.4 |
| 2.7 | FAIL | F18.2 |
| 2.8 | FAIL | F16.4, F18.4 |
| 2.9 | PARTIAL | F18.6 |
| 2.10 | FAIL | F18.5 |
| 2.11 | FAIL | F18.3 |
| 2.12 | FAIL | F18.1, F18.7 |
| 2.13 | FAIL | F3.2, F18.7 |
| 3.1 | PASS | F3.6, F4.7 |
| 3.2 | PASS | F4.3, F6.1 |
| 3.3 | FAIL | F4.1, F4.2, F6.2 |
| 3.4 | FAIL | F6.1–F6.6, F10.2 |
| 3.5 | FAIL | F5.1 |
| 3.6 | FAIL | F5.5 |
| 3.7 | PASS | F8.4 |
| 3.8 | FAIL | F10.1, F10.2, F10.8 |
| 3.9 | FAIL | F4.5, F4.6, F6.6, F8.4 |
| 3.10 | FAIL | F8.5, F10.4 |
| 3.11 | FAIL | F7.3, F8.1–F8.3, F8.7 |
| 3.12 | FAIL | F9.1–F9.4 |
| 3.13 | PARTIAL | F4.4 |
| 3.14 | PARTIAL | F8.6, F10.2, F10.3 |
| 4.1 | PARTIAL | F5.2, F5.4 |
| 4.2 | PASS | F5.3 |
| 4.3 | FAIL | F5.2 |
| 4.4 | PARTIAL | F5.3 |
| 4.5 | PASS | F5.3 |
| 4.6 | PARTIAL | F5.3 |
| 4.7 | FAIL | F5.3 (helper chosen over triggers, reason recorded) |
| 4.8 | FAIL | F5.4 |
| 4.9 | PARTIAL | F5.3 |
| 4.10 | PASS | F5.7 |
| 5.1 | PASS | F14.8 |
| 5.2 | PASS | F14.3 |
| 5.3 | PARTIAL | F10.2, F15.1–F15.3 |
| 5.4 | FAIL | F14.3 |
| 5.5 | FAIL | F14.4, F14.5 |
| 5.6 | PARTIAL | F14.6 |
| 5.7 | FAIL | F14.5 |
| 5.8 | FAIL | F14.1 |
| 5.9 | PARTIAL | F14.2 |
| 5.10 | FAIL | F14.7 |
| 5.11 | FAIL | F15.4 |
| 5.12 | FAIL | F7.1–F7.6 |
| 6.1 | PASS | F13.1, F13.8 |
| 6.2 | FAIL | F13.1 |
| 6.3 | PASS | F13.2 |
| 6.4 | PARTIAL | F12.6, F13.5 |
| 6.5 | FAIL | F13.3 |
| 6.6 | PARTIAL | F13.4 |
| 6.7 | PARTIAL | F13.2 |
| 6.8 | FAIL | F13.6 |
| 6.9 | FAIL | F7.2, F13.7 |
| 6.10 | PARTIAL | F12.6 |
| 7.1 | FAIL | F1–F18 (tests per task), F20.1 |
| 7.2 | FAIL | F1.1 |
| 7.3 | BELUM DIVERIFIKASI | F0.3, F20.1 |
| 7.4 | FAIL | F0.2, F20.5 |
| 7.5 | PARTIAL | F20.2 |
| 7.6 | FAIL | F18.3, F20.2 |
| 7.7 | FAIL | F20.6, F20.9 |

Totals: 16 PASS, 16 PARTIAL, 40 FAIL, 1 BELUM DIVERIFIKASI (matches the audit summary).

### 5.2 Findings (25)

| Finding | Severity | Task(s) |
|---|---|---|
| C-1 | Kritis | F7.3, F8.1–F8.3, F8.7, F9.1–F9.4 |
| T-1 | Tinggi | F4.1, F4.2, F4.3 |
| T-2 | Tinggi | F4.5, F8.4, F8.5, F8.6, F10.1–F10.4 |
| T-3 | Tinggi | F6.1–F6.6, F10.2, F10.5 |
| T-4 | Tinggi | F7.1–F7.6, F12.4, F12.7 |
| T-5 | Tinggi | F11.1–F11.6 |
| T-6 | Tinggi | F3.1–F3.4, F18.1–F18.7 |
| T-7 | Tinggi | F5.1, F5.4, F5.5 |
| T-8 | Tinggi | F10.2, F14.1–F14.7, F15.2, F15.3 |
| T-9 | Tinggi | F0.2, F0.5, §1.4, F20.5 |
| T-10 | Tinggi | F12.3, F13.1–F13.7 |
| S-1 | Sedang | F4.4 |
| S-2 | Sedang | F1.1 |
| S-3 | Sedang | F5.2, F5.3, F5.6 |
| S-4 | Sedang | F2.1–F2.4 |
| S-5 | Sedang | F1.3, F1.5, F2.4, F2.5, F17.1–F17.4 |
| S-6 | Sedang | F12.2, F12.3, F12.4 |
| S-7 | Sedang | F4.9, F8.9, F9.3 |
| S-8 | Sedang | F12.6, F12.7 |
| S-9 | Sedang | F12.8, F19.1 |
| R-1 | Rendah | F3.6 |
| R-2 | Rendah | F4.7 |
| R-3 | Rendah | F7.4 |
| R-4 | Rendah | F14.8 |
| R-5 | Rendah | F4.8 and every stage (§1.8) |

### 5.3 PRD v1.1 foundations in v0.1.0

| Requirement | Task(s) |
|---|---|
| `[brand]` mandatory (FR-1, FR-18, D-11) | F3.1–F3.4, F12.3, F14.11, F17.1, F18.3, F18.6 |
| RBAC schema and core checks (FR-6, D-9) | F3.1, F3.2, F8.3, F10.5 |
| Super role reads all, audited (FR-6, D-2 revised) | F9.1–F9.4 |
| Backup mandatory (FR-9, D-10) | F3.1, F3.2, F11 |
| Stability, maintainability, scalability (PRD §6, D-13) | F1.7, F3.1, F4.7, F5.8, F20.3 and every gate |
| Genericity (PRD §6, D10) | F2.7 |

## 6. PRD coverage

| PRD | Tasks |
|---|---|
| FR-1 `app.toml` | F3.1–F3.4, F12.1 |
| FR-2 Shell, navigation, design system | F14.1–F14.6, F14.8, F14.9 |
| FR-3 Settings registry | F14.7 |
| FR-4 Vault, recovery code, migrations, plain header | F4, F5.1, F5.5, F6 |
| FR-5 Sync-ready conventions | F5.2–F5.4 |
| FR-6 Profiles, PIN, roles, ownership | F8, F9, F10 |
| FR-7 AI assistant | F13 |
| FR-8 Pro, feedback, updater, crash | F12.3–F12.8 |
| FR-9 Backup and restore | F11 |
| FR-10 i18n and formatting | F15 |
| FR-11 Notes sample | F16 |
| FR-12 Generator | F18 |
| FR-13 Android | F19 |
| FR-14 Open-source hygiene | F1.3, F1.5, F2, F17 |
| NFR Security | F4, F6, F8, F9, F12.6 |
| NFR Privacy | F12.6, F13.3, F13.5 |
| NFR Performance | F20.3 |
| NFR Quality | F1, every gate |
| NFR Accessibility | F14.9, F19.3 |
| NFR Maintainability | F17.1 (`CHANGELOG.md` port notes), F20.6 |
| Success criteria 1–5 | 1: F18.7 · 2: F14.10, F19.4, F20.7 · 3: F16.5 · 4: F2 · 5: F5.4 |
| §9 Definition of done | F20.8 |

## 7. Original `task.md` → remediation tasks

| `task.md` | Remediation |
|---|---|
| S0 (S0.1–S0.4) | F0 (S0.2/S0.3 inventory against CATerm is out of scope here; record NOT VERIFIED if the CATerm repo is not available) |
| S1.2 | F2.3, F2.4 · S1.3: F20.6 |
| S2.1–S2.5 | F2.1–F2.5, F15.2 |
| S3.1–S3.4 | F3.1–F3.5 |
| S4.1–S4.7 | F14.1–F14.9 · S4.8: F15.4 |
| S5.1 | F4.8, F1.3 · S5.2: F5.1 · S5.3: F5.3 · S5.4: F5.4 · S5.5: F5.7 · S5.6: F5.5 · S5.7/S5.7b: F11 · S5.8: F12.6, F12.7 |
| S6.1 | F4 · S6.1b: F6 · S6.2: F8.6, F10.2 · S6.3: F8.4 · S6.4: F8.5, F10.4 · S6.5: F8.1–F8.3 · S6.6: F9 · S6.7: F10.1–F10.8 · S6.8: F8.8 · S6.9: F10.7 |
| S7.1–S7.5 | F13.1–F13.8 |
| S8.1–S8.4 | F12.1–F12.5, F12.8 |
| S9.1–S9.5 | F16.1–F16.6 |
| S10.1–S10.7 | F18.1–F18.8 |
| S11.1–S11.4 | F19.1–F19.5 |
| S12.1 | F1.5 · S12.2: F18.7 · S12.3: F17.1 · S12.4: F17.3 |
| S13.1–S13.2 | F17.2 (owner gate: F18.8) |
| S14.1–S14.3 | F20.4 |
| S15.1–S15.3 | F20.1, F20.6, F20.7, F20.9; S15.3 backlog: `Notes/backlog.md` written at F20.8 |

---

## Appendix A — Review of the earlier draft (`TASK_FIX.md` P0–P3)

Coverage of the 57 audit items that were not PASS: **35 fully covered, 10 partly, 12 not at all**. Coverage of the 25 findings: **10 fully, 7 partly, 8 not at all**.

**Not covered at all (items):** 2.4 module switches · 2.6 generated error prefix · 2.9 dry-run plan · 2.11 placeholder routes, menu i18n, stubs, `caframework_version` · 4.9 rollback test · 6.5 privacy levels and context providers · 6.6 guardrails per product · 6.7 language/tone/custom instructions · 6.10 crash scrubber email/card · 7.3 running the baseline commands · 7.5 release binary launch · 7.6 desktop launcher.

**Partly covered (items):** 1.5, 1.6, 1.7 (CommandPalette `hosts` mode, `api/performance.ts`, avatar preset, `release.yml` secret name, logo assets, docker-compose missing) · 2.3 (only `deny_unknown_fields`) · 3.9 (counter "in encrypted state" cannot work while locked) · 4.1 (`app_kv` exclusion missing) · 5.5 (`PageHeader` icon/accent missing) · 5.9 (groups, role filter, i18n labels missing) · 7.1 (no per-task tests) · 7.2 (clippy fixed only in P3).

**Findings not covered:** S-6, S-7, S-8, S-9, R-1, R-2, R-4, R-5. **Partly:** T-2, T-6, T-9, T-10, S-2, S-4, S-5.

**Design and ordering problems:** evidence folder created last (P3.6); clippy fixed last although it blocks every gate; capability checks (P0) before the `app.toml` roles they need (P2); migration runner (P2) after the key and schema changes (P0/P1); `CAF-VAULT-002` invented; scheduled backup without a key design; no acceptance criteria, verification commands or owner-verify path for GUI and Android checks.

| Draft item | Where it went |
|---|---|
| P0.1 | F7.3, F8.1–F8.3 |
| P0.2 | F9.1–F9.4 |
| P0.3 | F4.1–F4.3 (migration dropped per D-5) |
| P0.4 | F4.4 |
| P0.5 | F4.5, F4.6, F8.4 (counter outside the vault for the master password) |
| P1.1 | F7.1–F7.6 |
| P1.2 | F11.1–F11.6 |
| P1.3 | F8.4–F8.6, F10.1–F10.4 |
| P1.4 | F6.1–F6.6 |
| P2.1 | F5.1–F5.3 |
| P2.2 | F5.3, F5.6 |
| P2.3 | F5.4, F5.5 |
| P2.4 | F3.1–F3.4, F18.1–F18.7 |
| P3.1 | F14.1–F14.7 |
| P3.2 | F15.1–F15.4 |
| P3.3 | F12.6, F13.1–F13.7 |
| P3.4 | F2.1–F2.5 |
| P3.5 | F1.3, F1.5, F18.7 |
| P3.6 | F0.2, §1.4, F20.6, F20.9 |

