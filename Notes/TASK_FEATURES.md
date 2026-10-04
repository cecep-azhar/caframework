# CAFramework — Feature Task Plan v0.2.0 (`TASK_FEATURES.md` v1.0)

> **Status:** ready; starts after `TASK_FIX.md` F20 reaches `GATE_PASSED (agent)`. Decisions confirmed 4 October 2026.
> **Owner:** Cecep Azhar · **Inputs:** `prd.md` v1.1 (FR-1, FR-3, FR-6, FR-8, FR-9, FR-15–FR-19, §6) · `TASK_FIX.md` v2.1 · `prompt-dev.md` · `prompt-fix.md`
> **Run with:** `prompt-fix.md` (same rules, ledger and templates; stage IDs `G<n>`).
> **Language rule:** instructions in English; every report to the owner in Bahasa Indonesia.

## Ringkasan untuk pemilik

Rencana ini membangun fitur v0.2.0 di atas fondasi yang sudah dipasang di v0.1.0 (skema `app.toml` v1.1, `[brand]` wajib, pemeriksaan izin RBAC di core, backup wajib, dan syarat kualitas). Isinya 11 tahap:

| Tahap | Isi |
|---|---|
| G0 | Persiapan branch, ledger, dan baseline ukuran/kecepatan |
| G1 | Manajemen role saat runtime: super admin membuat role dan mengatur matriks izin |
| G2 | Connector registry untuk API apa pun, adapter GCC, dan antrian kirim offline |
| G3 | Scheduler (cron) dengan pengaturan jadwal, riwayat, tray, autostart, dan catch-up |
| G4 | Import/export data per entitas: CSV, XLSX, JSON, dengan pemetaan kolom dan pratinjau |
| G5 | Paket Pro yang selalu ada: paket, harga, lisensi, trial, dan fitur Free/Pro |
| G6 | Wizard di terminal (`cargo xtask wizard`), dengan branding Anda terisi otomatis |
| G7 | App Builder: wizard versi halaman di build dev CAFramework |
| G8 | Generator diperbarui untuk semua fitur baru, plus uji tiga aplikasi berbeda |
| G9 | Pengerasan kualitas: soak test 30 menit, fuzz, target performa, coverage |
| G10 | Android, re-audit read-only, dan kesiapan rilis v0.2.0 |

Aturan bukti sama dengan v0.1.0: tidak ada klaim tanpa perintah dan output asli, dan fitur yang hanya UI dilaporkan sebagai **UI SAJA**.

---

## 0. How to read and run this file

- Same conventions as `TASK_FIX.md` §0–§1: verification sets VS-R/F/G/A/E, done rules (§1.2), ratchets (§1.3), evidence (§1.4, now `Notes/evidence/feat/G<n>.md`), owner-verify protocol (§1.5), error rules (§1.8), dependencies (§1.9) and **quality bars (§1.10)**, which apply to every new line of code here.
- Branch: `feat/v0.2.0`, created from `fix/remediation-v2` at the F20 gate commit (or from the `v0.1.0` tag if the owner has created it).
- Every feature follows `prompt-dev.md` §6 (vertical slice) and is extensible through a **registry**, so products add entities, integrations, jobs, permissions and settings sections without editing framework files (PRD §6 maintainability).
- New test data uses fixture names and synthetic values only; no real contact data except the `[brand]` defaults.

## 1. Decisions applied (4 October 2026)

| ID | Decision | Stages |
|---|---|---|
| E-1 | v0.2.0 scope = PRD v1.1 features not delivered in v0.1.0 (PRD D19) | all |
| E-2 | RBAC: permission matrix, runtime role editing by the super role, generic default roles (PRD D11) | G1 |
| E-3 | Super role reads all data, audited (PRD D12) | G1, G4, G2 |
| E-4 | Connector registry + GCC built-in adapter (PRD D13) | G2 |
| E-5 | In-app scheduler, optional tray and start-at-login, catch-up (PRD D14) | G3 |
| E-6 | Backup mandatory; per-entity import/export CSV/XLSX/JSON (PRD D15) | G3, G4 |
| E-7 | `[brand]` mandatory; owner's details are wizard defaults (PRD D16) | G6, G7, G8 |
| E-8 | Pro package always generated, Free/Pro gating, payments on GCC/Mayar.id, teams/sync deferred (PRD D17) | G5 |
| E-9 | Wizard CLI + App Builder GUI on one schema (PRD D18) | G6, G7 |
| E-10 | Stable, maintainable, scalable as measurable bars (PRD §6, D20) | G9 and every gate |

**External dependency:** GCC must implement the endpoints in `docs/backend-contract.md` v2 (G2.6). Until then every GCC feature is tested against the mock server and marked **BELUM DIVERIFIKASI (server GCC)** for the real call.

## 2. Stage map

| Stage | Title | PRD | Depends on | Runner | Size |
|---|---|---|---|---|---|
| G0 | Baseline v0.2.0 | §6 | F20 | any | S |
| G1 | Roles & permissions at runtime | FR-6, FR-3 | G0 | any + owner smoke | M |
| G2 | Integrations and GCC adapter | FR-15, FR-8 | G1 | any | L |
| G3 | Scheduler, tray, start at login | FR-16, FR-9 | G2 | any + owner smoke | L |
| G4 | Data import/export per entity | FR-17 | G3 | any | L |
| G5 | Pro package and feature gating | FR-8 | G4 | any + owner smoke | L |
| G6 | Wizard CLI | FR-19, FR-18 | G5 | any | M |
| G7 | App Builder GUI | FR-19 | G6 | any + owner smoke | M |
| G8 | Generator for v1.1 and three-app E2E | FR-12, success criterion 6 | G7 | any (+ x1-bench) | M |
| G9 | Quality hardening | §6 | G8 | any + x1-bench | M |
| G10 | Android, QA re-audit, release readiness v0.2.0 | FR-13, §9 | G9 | x1-bench | M |

---

### G0 — Baseline v0.2.0

**PRD:** §6 · **Runner:** any · **Depends on:** F20 · **Size:** S

- [ ] **G0.1 Branch and ledger.**
  **Do:** create `feat/v0.2.0` from the F20 gate commit (record hash); add a `## v0.2.0` section to `Notes/fix-ledger.md` with G0–G10 tasks as `TODO`, the E-decisions table and an empty OWNER-VERIFY queue; create `Notes/evidence/feat/` and `Notes/evidence/feat/logs/`; copy this file to `Notes/TASK_FEATURES.md`.
  **Accept:** files committed; ledger shows F20 status and G0 `IN_PROGRESS`.
  **Verify:** `git log -1 --format='%H %s'` · `ls Notes/evidence/feat`.

- [ ] **G0.2 Baseline measurements.**
  **Do:** run VS-R, VS-F, VS-G, VS-E (and VS-A if available); record coverage (core/frontend), bench results (`cargo bench -p caf-core -- --save-baseline g0`), cold start (if x1-bench), file-size and docs ratchets.
  **Accept:** all green; baseline table in `G0.md`.
  **Verify:** logs.

**Gate G0:** green baseline recorded.

---

### G1 — Roles & permissions at runtime

**PRD:** FR-6, FR-3, D11, D12 · **Runner:** any + owner smoke · **Depends on:** G0 · **Size:** M

- [ ] **G1.1 Role management in core.**
  **Do:** commands `list_roles`, `create_role`, `update_role` (labels, permission set), `delete_role` (only when no profile uses it; otherwise `RbacError::RoleInUse`), `assign_role(profile_id, role_key)`, `permission_catalogue()`; all need `roles:manage` except `list_roles`/`permission_catalogue` (`roles:view`); writes go through `sync.rs`; only allowed when `[rbac].allow_runtime_roles`. Invariants enforced in core: super role permissions fixed to `*` and not deletable or renamable; at least one profile keeps the super role; a profile cannot remove `roles:manage` from its own role if that would leave no profile able to manage roles; permissions must exist in the build-time catalogue (`RbacError::UnknownPermission`).
  **Accept:** tests for every invariant, wildcard expansion, and that a session's permissions update immediately after its role changes (next command uses the new set).
  **Verify:** `cargo test -p caf-core rbac:: -- --nocapture`.

- [ ] **G1.2 Audit of role changes.**
  **Do:** audit events `ROLE_CREATED`, `ROLE_UPDATED` (diff of permissions), `ROLE_DELETED`, `ROLE_ASSIGNED`; `list_audit_events` (paginated, `audit:view`).
  **Accept:** test reads events with correct diffs; pagination test.
  **Verify:** `cargo test -p caf-core audit::roles`.

- [ ] **G1.3 Roles & Permissions settings section.**
  **Do:** registry section (`roles:view`): role list with labels and member counts; editor with a matrix (rows = resources grouped as Menus / Data / System, columns = actions, cells = checkbox; "all" per row and column; wildcard shown as "semua"); create/duplicate/delete role; assign roles from the profile list; super role row locked with an explanation; unsaved-changes guard; diff preview before saving. Audit log viewer (`audit:view`) with filters.
  **Accept:** wiring chain per action; vitest tests for matrix logic (wildcard collapse/expand, locked super role); Playwright screenshots both themes, 1280 and 390 px.
  **Verify:** `npm run test -- roles` · screenshots.

- [ ] **G1.4 Permission-driven UI everywhere.**
  **Do:** one helper `can(permission)` from the session's permission set (fetched after `select_profile` and on `roles-changed` events) drives nav, palette, settings sections and action buttons; no role-name checks in the frontend (`git grep` guard: no `'super_admin'|'admin'` literals outside generated files).
  **Accept:** guard negative fixture; vitest tests render the shell for each default role and compare visible menus with the matrix.
  **Verify:** `cargo run -p caf-xtask -- guard --only role_literals` · `npm run test -- can`.

- [ ] **G1.5 Owner smoke test.**
  **Do:** OWNER-VERIFY: create role "Kasir" with `notes:view,create`, assign to a profile, switch profile, confirm menus and buttons match, try a forbidden action through DevTools `invoke` (expect `PermissionDenied`).
  **Accept:** owner output recorded.
  **Verify:** OWNER-VERIFY ID.

**Gate G1:** VS green; invariants proven; UI driven only by permissions.

---

### G2 — Integrations and GCC adapter

**PRD:** FR-15, FR-8 · **Runner:** any · **Depends on:** G1 · **Size:** L

- [ ] **G2.1 Connector registry.**
  **Do:** `CORE/integrations/` with `IntegrationSpec` generated from `[[integrations]]` and an `Adapter` trait for typed product clients; `ConnectorClient::request(key, method, path, query, body, opts)` built on the F12.3 HTTP client; per-integration timeout, retries (exponential back-off with jitter, only for idempotent methods or requests carrying an idempotency key), concurrency limit (default 4) and rate limit (token bucket); `allowed_hosts` check on every request and redirect; responses size-capped (default 10 MB).
  **Accept:** mock-server tests for retry policy (no retry on POST without key), rate limit, host allow-list (redirect to another host refused), size cap; property test for URL joining.
  **Verify:** `cargo test -p caf-core integrations::client`.

- [ ] **G2.2 Authentication methods.**
  **Do:** `none`, `api_key_header`, `api_key_query`, `bearer`, `basic`, `hmac_sha256` (canonical string `METHOD\nPATH\nTIMESTAMP\nSHA256(body)`, headers `X-Signature`, `X-Timestamp`; documented), `oauth2_client_credentials` (token cached in memory with expiry, refreshed 60 s early). Secrets stored in the vault (`integration_secrets` table, excluded from the sync guard with reason) via `set_integration_secret` (`integrations:manage`); `get_integration_status` returns only `{ configured, last_test_at, last_error_code }`.
  **Accept:** one mock-server test per auth method asserting the exact headers/query; test that no IPC response type contains a secret (serialize all integration DTOs with a planted secret and grep).
  **Verify:** `cargo test -p caf-core integrations::auth`.

- [ ] **G2.3 Outbox for offline writes.**
  **Do:** table `outbox(id, integration, method, path, body, idempotency_key, attempts, next_attempt_at, last_error, created_at, status)`; `enqueue` used by adapters for writes; flush job (G3) sends in order per integration, backs off, marks `failed` after 10 attempts and notifies; super role can retry or discard from the UI.
  **Accept:** tests: offline → queued; online → sent once (mock asserts idempotency key); ordering per integration; failure after 10 attempts.
  **Verify:** `cargo test -p caf-core integrations::outbox`.

- [ ] **G2.4 Request log with redaction.**
  **Do:** ring buffer (last 200 requests per integration) with method, host, path, status, duration, error code; headers and bodies never stored; query values of auth parameters masked; PII scrubber (F12.6) applied to error messages.
  **Accept:** test plants a key in a query string and asserts it is masked.
  **Verify:** `cargo test -p caf-core integrations::log`.

- [ ] **G2.5 Integrations settings section.**
  **Do:** list integrations with status; per integration: base URL (if `user_configurable`), secret fields by auth method (write-only inputs), "Test connection", recent requests, outbox view; permissions `integrations:view|manage`.
  **Accept:** wiring chain; screenshots.
  **Verify:** vitest tests · screenshots.

- [ ] **G2.6 GCC adapter and contract v2.**
  **Do:** `CORE/integrations/gcc.rs` covers: device registration, licence activation/refresh/deactivation, plans and prices, hosted AI proxy, feedback, crash reports, update manifest pointer, generic data push/pull (`POST /v1/apps/{app_id}/data/{entity}` with batches, `GET …?since=<cursor>`), server time. Every call sends `X-CA-App-Id`, `X-CA-App-Version`, `X-CA-Device-Id` and is HMAC-signed with the device secret issued at registration. Write `docs/backend-contract.md` v2: endpoints, payloads, error codes, versioning (`/v1`), rate limits, idempotency, signature scheme, and a JSON Schema per payload under `docs/contracts/`.
  **Accept:** mock GCC server (in `crates/caf-core/tests/support/mock_gcc.rs`) used by tests for every endpoint; contract schemas validated against the mock responses in tests; real server call **BELUM DIVERIFIKASI (server GCC)** with an OWNER-VERIFY entry listing what GCC must implement.
  **Verify:** `cargo test -p caf-core --test gcc_contract`.

- [ ] **G2.7 Adding an integration (docs + example).**
  **Do:** `docs/adding-an-integration.md`; example REST integration (public, no key needed, mocked in tests) registered only in the sample.
  **Accept:** doc steps reproduced in a temp copy by a test that adds an integration from a fixture `app.toml` without editing framework files (`git diff --stat` of framework dirs empty).
  **Verify:** `cargo test -p caf-xtask --test add_integration`.

**Gate G2:** VS green; every auth method and the GCC contract tested against mocks; no secret reaches the frontend.

---

### G3 — Scheduler, tray, start at login

**PRD:** FR-16, FR-9 · **Runner:** any + owner smoke · **Depends on:** G2 · **Size:** L

- [ ] **G3.1 Job registry and schedule parsing.**
  **Do:** `CORE/scheduler/`: `JobSpec { key, title_key, default_schedule, permission, requires_unlocked, catch_up: Skip | RunOnce, max_runtime, run }`; registry filled by modules and products (one file for products); schedules: 5-field cron (crate `croner` or `cron`, licence check) in local time, presets, and `every <n> minutes` (min 5); next-run calculation across DST changes; human-readable description in `id` and `en` ("Setiap Senin pukul 02.00" / "Every Monday at 02:00").
  **Accept:** tests for parsing, next-run times (including month ends and DST using a fixed TZ fixture), description strings; property test that next-run is always in the future.
  **Verify:** `cargo test -p caf-core scheduler::schedule`.

- [ ] **G3.2 Runtime.**
  **Do:** one scheduler task in the app process with an injected clock; tables `scheduled_jobs(key, enabled, schedule, last_run_at, next_run_at, last_status)` and `job_runs(seq, job_key, started_at, finished_at, status, message)` (retention 500, configurable); one instance per job (overlap → `skipped_overlap`); `max_runtime` cancellation; panics inside a job are caught (`catch_unwind` around the job future) and recorded as `failed` without crashing the app; jobs needing the vault wait while locked (`waiting_unlock`) and run on unlock; catch-up on start per policy; `run_job_now(key)` (`scheduler:run`).
  **Accept:** fake-clock tests for each state, overlap, timeout, panic isolation, catch-up `Skip` vs `RunOnce`, retention pruning.
  **Verify:** `cargo test -p caf-core scheduler::runtime`.

- [ ] **G3.3 Built-in jobs.**
  **Do:** migrate the F11 backup schedule onto the scheduler (`backup.scheduled`, keeping its settings); add `backup.overdue_reminder` (daily), `integrations.outbox_flush` (every 5 min), `pro.licence_refresh` (daily, G5), `audit.prune` (weekly, keeps 180 days).
  **Accept:** tests that each built-in job is registered with its default schedule; backup schedule data from F11 migrates without loss (migration test).
  **Verify:** `cargo test -p caf-core scheduler::builtin`.

- [ ] **G3.4 Tray and start at login (desktop).**
  **Do:** Tauri tray icon (open, lock now, run backup now, quit); "close to tray" and "start at login" settings, both off by default (`tauri-plugin-autostart`); when the window is closed to tray the scheduler keeps running and auto-lock still applies; a notification explains the first close-to-tray.
  **Accept:** VS-A build; OWNER-VERIFY on x1-bench: enable both, log out/in, confirm the app starts hidden, the backup job runs at a near schedule, tray menu works.
  **Verify:** `cargo test -p caf-app tray` (VS-A) · OWNER-VERIFY ID.

- [ ] **G3.5 Schedules settings section.**
  **Do:** list jobs with next run, last status, enabled toggle; editor with presets, cron field with live validation and description, "run now", run history with messages; permissions `scheduler:view|manage|run`.
  **Accept:** wiring chain; screenshots.
  **Verify:** vitest tests · screenshots.

- [ ] **G3.6 Android behaviour.**
  **Do:** jobs run while the app is in the foreground plus catch-up on resume; document limits and the WorkManager option in `docs/android.md` (implement only if a maintained Tauri 2 plugin exists; otherwise record as a gap).
  **Accept:** doc section; G10 on-device check includes a catch-up run.
  **Verify:** file content.

- [ ] **G3.7 Adding a job (docs).**
  **Do:** `docs/adding-a-job.md` with a product example registered from one file.
  **Accept:** test adds a job from a fixture without editing framework files.
  **Verify:** `cargo test -p caf-xtask --test add_job`.

**Gate G3:** VS green; scheduler survives failing and panicking jobs; backup runs on the scheduler.

---

### G4 — Data import/export per entity

**PRD:** FR-17 · **Runner:** any · **Depends on:** G3 · **Size:** L

- [ ] **G4.1 Entity registry.**
  **Do:** `EntitySpec` generated from `[[entities]]` plus a Rust field description per entity (`FieldSpec { name, kind: Text|Int|Decimal|Bool|Date|DateTime|Enum|Json, required, unique, max_len, enum_values, label_key }`), natural key, formats, permissions `<entity>:import|export`. Notes is registered as the sample.
  **Accept:** arch test: every `[[entities]]` key has a field spec and a sync table; negative fixture.
  **Verify:** `cargo test -p caf-core --test arch`.

- [ ] **G4.2 Readers and writers.**
  **Do:** CSV (UTF-8 with or without BOM, delimiter auto-detect `,`/`;`/tab, quoted fields), XLSX read (`calamine`) and write (`rust_xlsxwriter`), JSON (array of objects with `schema_version`); streaming reads; limits 50 000 rows / 20 MB (configurable per entity, errors `ImportError::TooLarge`); export neutralises formula injection (prefix `'` for cells starting with `=`, `+`, `-`, `@`, tab, CR); dates in ISO 8601, decimals with `.` in files and locale formatting only in the UI.
  **Accept:** round-trip tests per format; formula-injection test; fuzz/property tests on the CSV and XLSX readers (no panic, ≥ 10 000 cases in CI); licences recorded.
  **Verify:** `cargo test -p caf-core data_io::formats`.

- [ ] **G4.3 Import pipeline.**
  **Do:** `import_preview(entity, file)` → detected columns, automatic mapping (by field name and both locale labels, case/space-insensitive), first 50 rows with per-cell validation; `import_execute(entity, file, mapping, mode: InsertOnly | UpsertById | UpsertByNaturalKey, on_error: AbortAll | SkipInvalid, dry_run)`; one transaction; writes through `sync.rs` with the importer as owner and the entity's default visibility; result `{ inserted, updated, skipped, failed, report_file }` where the report is a CSV of failed rows with reasons (i18n); progress events every 1 000 rows; cancellable.
  **Accept:** tests for each mode and error policy; abort leaves the DB unchanged; upsert respects visibility (cannot update another profile's private row unless super role, audited); 50 000-row import ≤ 30 s on the reference runner (bench).
  **Verify:** `cargo test -p caf-core data_io::import` · `cargo bench -p caf-core import`.

- [ ] **G4.4 Export and templates.**
  **Do:** `export(entity, format, filter)` within the session's visibility and permissions (super role includes others' private rows, audited); `download_template(entity, format)` with headers in the active locale and a hidden machine header row (XLSX) or a comment line (CSV).
  **Accept:** tests: member export excludes others' private rows; template re-imports without mapping changes; 100 000-row export ≤ 15 s (bench).
  **Verify:** `cargo test -p caf-core data_io::export`.

- [ ] **G4.5 Data Import/Export UI.**
  **Do:** settings section plus an "Import/Export" action on every entity page: stepper (file → mapping → preview → options → run → report), progress bar, cancel, report download; four states; i18n.
  **Accept:** wiring chain; Playwright run of the full stepper with a fixture CSV against the mocked bridge; screenshots.
  **Verify:** `npx playwright test import` · screenshots.

- [ ] **G4.6 Adding an entity (docs).**
  **Do:** `docs/adding-an-entity.md` (migration, field spec, registry entry, permissions, import/export for free).
  **Accept:** test adds an entity from a fixture without editing framework files and imports a CSV into it.
  **Verify:** `cargo test -p caf-xtask --test add_entity`.

**Gate G4:** VS green; performance budgets met; fuzz suites green.

---

### G5 — Pro package and feature gating

**PRD:** FR-8, D17 · **Runner:** any + owner smoke · **Depends on:** G4 · **Size:** L

- [ ] **G5.1 Entitlements in core.**
  **Do:** `CORE/pro/entitlements.rs`: feature catalogue from `[pro.features]`; `require(feature)` → `ProError::FeatureLocked { feature, plan_needed }`; state machine `Free → Trial(expires) → Pro(expires, devices) → Grace(until) → Free`; licence token = GCC-signed (Ed25519) JSON with `app_id`, `device_id`, plan, features, `expires_at`, `issued_at`; verified offline with the public key embedded at build time (`<CODE>_PRO_LICENSE_PUBKEY_V1`); stored in the vault; clock-rollback detection (stored `max_seen_time`); `offline_grace_days` after `expires_at` without a successful refresh. Debug builds only: `CAF_PRO_DEV_UNLOCK=1` unlocks all features with a visible "DEV" badge (compiled out of release builds; test proves it).
  **Accept:** tests for every transition, tampered token, wrong `app_id`/`device_id`, clock rollback, grace expiry; property test on token parsing; release-build test that the dev switch has no effect.
  **Verify:** `cargo test -p caf-core pro:: -- --nocapture` · `cargo test -p caf-core --release pro::dev_switch`.

- [ ] **G5.2 Gating applied.**
  **Do:** policy registry gains an optional `feature` field; `guarded` checks it after permissions; framework features mapped: `ai_assistant`, `scheduled_backup`, `data_export_xlsx`, `integrations_custom` (products may change the mapping in `app.toml`).
  **Accept:** table-driven test over gated commands × {Free, Trial, Pro, Grace, expired}.
  **Verify:** `cargo test -p caf-core pro::gating`.

- [ ] **G5.3 GCC flows.**
  **Do:** using the G2.6 adapter: fetch plans and prices (cached 24 h), start trial, activate with licence key or account login, refresh (daily job), deactivate device, list devices; checkout opens `GCC/Mayar.id` URL in the browser via `open_external_url` with `app_id` and `device_id` parameters; after payment the app polls `licence_refresh` for 10 minutes or until active.
  **Accept:** mock GCC tests for each flow, including device-limit reached and payment pending; real flow **BELUM DIVERIFIKASI (server GCC)**.
  **Verify:** `cargo test -p caf-core --test gcc_contract pro_`.

- [ ] **G5.4 Pro UI.**
  **Do:** Pro page (plans, prices in IDR via `formatCurrency`, feature comparison table from `[pro.features]`, current status, trial button, activate, manage devices); `<ProGate feature>` component wrapping gated UI with an upgrade prompt; status badge in title bar and About; lock-screen login mode only when Pro is configured.
  **Accept:** vitest tests for `ProGate` in every state; screenshots both themes.
  **Verify:** `npm run test -- pro` · screenshots.

- [ ] **G5.5 Always generated.**
  **Do:** `[modules].pro` defaults to `true`; the wizard always writes a `[pro]` section; the framework `app.toml` switches `pro = true` with empty `gcc_product_id` (features locked with an explanation).
  **Accept:** generator test: output without explicit `pro` key has the Pro module and page.
  **Verify:** `cargo test -p caf-xtask --test pro_default`.

- [ ] **G5.6 Owner smoke test.**
  **Do:** OWNER-VERIFY with the mock GCC (`cargo run -p caf-core --example mock_gcc`): trial → expire (time travel via mock) → grace → locked → activate → unlocked.
  **Accept:** owner output recorded.
  **Verify:** OWNER-VERIFY ID.

**Gate G5:** VS green; entitlement state machine proven; gating enforced in core.

---

### G6 — Wizard CLI

**PRD:** FR-19, FR-18 · **Runner:** any · **Depends on:** G5 · **Size:** M

- [ ] **G6.1 Template library crate.**
  **Do:** extract config types, validation, codegen and new-app logic from `caf-xtask` into a library crate `caf-template` (no GUI deps) used by `caf-xtask`, the wizard and the App Builder; derive `schemars::JsonSchema` for all config types and generate `schema/app.schema.json`.
  **Accept:** `caf-xtask` behaviour unchanged (all F3/F18 tests green); schema file reproducible (`codegen --check`).
  **Verify:** VS-R · VS-G.

- [ ] **G6.2 Brand defaults file.**
  **Do:** `~/.config/caframework/brand.toml` created on first wizard run with the owner defaults (D-11) and editable via `cargo xtask wizard --edit-brand`; the wizard pre-fills `[brand]` from it.
  **Accept:** test with a temp `XDG_CONFIG_HOME`: first run creates the file with the defaults; edits persist.
  **Verify:** `cargo test -p caf-xtask --test wizard_brand`.

- [ ] **G6.3 Interactive flow.**
  **Do:** `cargo xtask wizard [--out app.toml] [--generate <dir>] [--with-sample]` using `inquire` (licence check): steps identity (name → suggested slug/code/identifier, editable), brand (pre-filled, confirm), accent (colour swatches), locales and default, platforms, modules (backup shown as mandatory), roles (presets: "Pengguna tunggal", "Tim: super_admin/admin/member/viewer", "Kustom") and per-role permission presets, menus (key, labels, icon chosen from a bundled 24×24 icon set by name, group, permission), entities (fields), integrations (GCC yes/no, add REST), jobs (built-ins + schedules), Pro features (free/pro per feature); each step validates immediately with the shared validator; review screen shows the full `app.toml` and a summary; Back on every step; Ctrl-C leaves nothing written.
  **Accept:** snapshot tests of each step using `inquire`'s test backend or a scripted terminal (`expectrl`); interrupted run writes no file.
  **Verify:** `cargo test -p caf-xtask --test wizard_interactive`.

- [ ] **G6.4 Non-interactive mode.**
  **Do:** `--answers answers.toml` (schema in `schema/answers.schema.json`) runs the same steps without prompts; errors list every invalid answer.
  **Accept:** three answer fixtures (single-user notes app; team app with four roles; finance app with Pro features and two integrations) produce valid `app.toml` files identical to golden files.
  **Verify:** `cargo test -p caf-xtask --test wizard_answers`.

- [ ] **G6.5 Docs.**
  **Do:** `docs/creating-an-app.md` starts with the wizard; `docs/wizard.md` documents every step and the answers file.
  **Accept:** link check clean.
  **Verify:** link checker output.

**Gate G6:** VS green; wizard produces valid configs interactively and from answers.

---

### G7 — App Builder GUI

**PRD:** FR-19 · **Runner:** any + owner smoke · **Depends on:** G6 · **Size:** M

- [ ] **G7.1 Builder backend.**
  **Do:** behind a Cargo feature `builder` enabled only in CAFramework's dev builds (never in generated apps or release builds): commands `builder_schema`, `builder_validate(config)`, `builder_preview(config)` (nav items, accent classes, permission matrix), `builder_generate(config, out_dir, with_sample)` calling `caf-template`; output dir chosen through the dialog plugin and checked like `new-app`.
  **Accept:** tests: release build has no builder commands (`generate_handler!` inspection in a test); validation identical to CLI for the three G6.4 fixtures.
  **Verify:** `cargo test -p caf-app --features builder builder::` (VS-A) · `cargo test -p caf-app --release no_builder`.

- [ ] **G7.2 Builder page.**
  **Do:** dev-only route `/_dev/builder` with the same steps as the wizard as a form (JSON-Schema-driven validation in the browser with `ajv`, final validation by `builder_validate`), live preview panel (sidebar with icons and groups, accent colours, role × permission matrix, lock-screen brand footer), "download app.toml", "generate"; brand pre-filled from `brand.toml` via a command; excluded from production builds.
  **Accept:** Playwright test completes the form for one fixture and compares the downloaded `app.toml` with the golden file; `ls frontend/build | grep -c _dev` = 0.
  **Verify:** `npx playwright test builder` · the command.

- [ ] **G7.3 Owner smoke test.**
  **Do:** OWNER-VERIFY: `cargo tauri dev --features builder`, build an app with 3 menus and 2 roles, generate into a temp dir, run VS-F inside it, launch it.
  **Accept:** owner output recorded.
  **Verify:** OWNER-VERIFY ID.

**Gate G7:** VS green; GUI and CLI produce identical configs.

---

### G8 — Generator for v1.1 and three-app E2E

**PRD:** FR-12, success criterion 6 · **Runner:** any (+ x1-bench) · **Depends on:** G7 · **Size:** M

- [ ] **G8.1 Generator covers every v1.1 section.**
  **Do:** `new-app` generates permission catalogue, role seeds, entity registrations (with placeholder migrations and field specs marked `UI-ONLY placeholder` until the product fills them), integration registrations, job registrations, Pro feature map, brand files and installer metadata; the sample-strip logic covers every Notes registration (entity, AI provider, job if any).
  **Accept:** tree listing and greps for each registry in a generated app.
  **Verify:** `cargo test -p caf-xtask --test generate_v11`.

- [ ] **G8.2 Three-app E2E (success criterion 6).**
  **Do:** extend VS-E: generate the three G6.4 fixtures through `wizard --answers … --generate`, run VS-R, VS-F, VS-G in each (VS-A on x1-bench), run `guard --only genericity` and the residue check in each.
  **Accept:** full logs; all green.
  **Verify:** `cargo test -p caf-xtask --test new_app_e2e -- --ignored --nocapture`.

- [ ] **G8.3 Port notes.**
  **Do:** `CHANGELOG.md` v0.2.0 entry with port notes per feature for apps generated from v0.1.0; `docs/porting-framework-fixes.md` updated.
  **Accept:** entries exist for G1–G7.
  **Verify:** file content.

**Gate G8:** VS-E green for all three apps.

---

### G9 — Quality hardening

**PRD:** §6 (stability, maintainability, scalability) · **Runner:** any + x1-bench · **Depends on:** G8 · **Size:** M

- [ ] **G9.1 Soak test.**
  **Do:** 30-minute headless soak (`crates/caf-core/tests/soak.rs`, `#[ignore]`) with an accelerated fake clock plus real wall time: scheduler running all built-in jobs every minute, repeated imports/exports of 10 000 rows, auto-lock/unlock cycles, outbox flush against a flaky mock server (20 % errors); sample RSS every 30 s.
  **Accept:** zero unexpected errors; RSS growth ≤ 10 % between minute 5 and minute 30; log attached.
  **Verify:** `cargo test -p caf-core --test soak -- --ignored --nocapture`.

- [ ] **G9.2 Fuzzing.**
  **Do:** `cargo fuzz` targets for `app.toml` parsing, backup header parsing, keyring JSON, licence token, CSV and XLSX readers; run each 10 minutes on x1-bench or CI nightly; corpus committed under `fuzz/corpus` (small files only).
  **Accept:** no crash; run stats attached.
  **Verify:** `cargo fuzz run <target> -- -max_total_time=600` logs.

- [ ] **G9.3 Performance budgets.**
  **Do:** run the full bench suite on the 100 000-row dataset: list page, search, import 50 000, export 100 000, backup, scheduler tick; compare with G0 baseline; CI job fails on > 20 % regression.
  **Accept:** every PRD §6 budget met on x1-bench; table attached.
  **Verify:** `cargo bench -p caf-core -- --baseline g0`.

- [ ] **G9.4 Maintainability targets.**
  **Do:** reach coverage core ≥ 80 %, frontend ≥ 70 %; `missing_docs` ratchet empty; file-size allow-list reviewed (each exception justified or removed); ADR index `docs/adr/README.md`.
  **Accept:** coverage report; ratchets.
  **Verify:** `cargo llvm-cov --workspace --exclude caf-app --summary-only` · `npx vitest run --coverage`.

**Gate G9:** all PRD §6 bars enforced and green.

---

### G10 — Android, QA re-audit, release readiness v0.2.0

**PRD:** FR-13, §9 · **Runner:** x1-bench · **Depends on:** G9 · **Size:** M

- [ ] **G10.1 Android.**
  **Do:** rebuild APK; on-device OWNER-VERIFY: roles matrix, integrations settings, schedules (catch-up on resume), import of a CSV from device storage, Pro page, brand footer; touch-target check (F19.3 harness) on new screens.
  **Accept:** owner output; 0 touch-target violations.
  **Verify:** OWNER-VERIFY ID · `npx playwright test mobile`.

- [ ] **G10.2 Read-only QA re-audit.**
  **Do:** separate session with `prompt-fix.md` Appendix B extended to the v1.1 features (G-stages listed in this file); fix loop `G10.R<n>` until 0 critical, 0 high, 0 UI-ONLY.
  **Accept:** final re-audit file.
  **Verify:** `Notes/audit/<date>-reaudit-v020.md`.

- [ ] **G10.3 Version, changelog, DoD.**
  **Do:** version `0.2.0` in all manifests; DoD table for PRD §9 v0.2.0 with evidence links; prepare (not create) tag `v0.2.0`; owner actions listed (merge, tag, publish).
  **Accept:** table complete; open items listed honestly.
  **Verify:** `git grep -n '0.2.0' -- Cargo.toml frontend/package.json crates/caf-app/tauri.conf.json`.

**Gate G10:** DoD v0.2.0 table complete with evidence; owner actions recorded.

---

## 3. PRD v1.1 coverage

| PRD | Tasks |
|---|---|
| FR-1 (v1.1 sections) | v0.1.0 F3; G8.1 |
| FR-3 new sections | G1.3, G2.5, G3.5, G4.5 |
| FR-6 RBAC runtime, super role | v0.1.0 F8.3, F9; G1.1–G1.5 |
| FR-8 Pro package | G5.1–G5.6 |
| FR-9 backup on scheduler | G3.3 |
| FR-12 generator v1.1 | G8.1, G8.2 |
| FR-15 integrations | G2.1–G2.7 |
| FR-16 scheduler | G3.1–G3.7 |
| FR-17 import/export | G4.1–G4.6 |
| FR-18 branding | v0.1.0 F3, F14.11, F17.1; G6.2, G7.2 |
| FR-19 wizard and App Builder | G6.1–G6.5, G7.1–G7.3 |
| §6 stability | G2.1 (limits), G3.2 (job isolation), G9.1, G9.2 |
| §6 maintainability | registries in G1–G4, G6.1, G9.4 |
| §6 scalability | G2.1 (concurrency), G4.3/G4.4 (budgets), G9.3 |
| Success criterion 6 | G6.4, G8.2 |
| §9 v0.2.0 DoD | G10.3 |
