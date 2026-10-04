# CAFramework — Product Requirements Document (v1.1)

> Status: v1.0 approved 1 October 2026; **v1.1 amendments approved 4 October 2026** (marked "v1.1"). Owner: Cecep Azhar.
> Companion files: `task.md` (original stages), `TASK_FIX.md` (remediation → v0.1.0), `TASK_FEATURES.md` (new features → v0.2.0), `prompt-dev.md`, `prompt-fix.md`.
> Language rule: this document and all agent instructions are in English; **all agent reports to the owner are in Bahasa Indonesia**.

## Ringkasan untuk pemilik

CAFramework adalah **starter template open-source (MIT)** yang dibuat dari CATerm tanpa semua bagian SSH. Framework ini **generik**: tidak terikat satu produk, sehingga bisa dipakai untuk aplikasi keuangan, kantor, inventori, kasir, catatan, dan lainnya. Isinya: shell aplikasi, navigasi, auth (vault + lock screen + multi-profil PIN), **RBAC dengan super admin dan matriks izin**, Settings, tema, i18n ID/EN, vault terenkripsi, error code, crash report, **backup wajib**, **import/export data per entitas**, updater, Asisten AI, **integrasi API dengan adapter GCC**, **penjadwal (cron)**, **paket Pro yang selalu tersedia**, dan **branding pengembang yang wajib**. Tampilannya identik dengan CATerm.

Aplikasi baru dibuat lewat **wizard** (`cargo xtask wizard` di terminal, atau halaman App Builder di build dev), yang menulis `app.toml` lalu menjalankan `cargo xtask new-app`. Hasilnya repo baru yang bebas diubah. Framework menyertakan menu contoh "Catatan" yang tersambung dari UI sampai DB terenkripsi.

Semua fitur wajib **stabil, mudah dirawat, dan scalable**, dengan ambang terukur di §6.

**Pembagian rilis.** v0.1.0 menutup temuan audit dan memasang fondasi v1.1: skema `[brand]` dan RBAC, backup wajib, dan aturan super role. v0.2.0 menambah manajemen role saat runtime, integrasi API, scheduler, import/export entitas, paket Pro dengan pembatasan fitur, dan wizard.

---

## 1. Problem and goal

Every product the owner builds (CACash, CAPost, CANet, CADev, CACode Kids, and future office, inventory or point-of-sale apps) needs the same foundation CATerm already has: Tauri shell, design system, navigation, encrypted storage, lock screen, settings, i18n, error taxonomy, crash reporting, updater, licensing, AI. Rebuilding or hand-copying it per product causes drift and repeated security bugs.

**Goal:** a reusable, product-agnostic, fully customisable starter template from which a new product skeleton can be generated in minutes through a wizard, and that already proves the full feature chain works.

**Success criteria (measurable):**

1. `cargo xtask new-app --config app.toml --out <dir>` (or the wizard) produces a repo that passes all verification commands with no manual edits.
2. The generated app launches on Linux and Android, looks structurally identical to CATerm (owner judgement on screenshots), and contains one placeholder page per configured menu.
3. The sample "Notes" menu passes the wiring proof: create in UI → restart app → read back from the encrypted DB.
4. Zero SSH/SFTP/terminal code or dependencies remain in the framework.
5. Every table created through the framework satisfies the sync-ready data conventions (enforced by a test).
6. **v1.1:** three structurally different apps (for example a single-user notes app, a team app with four roles, a finance app with Pro features and two integrations) are generated from the wizard's non-interactive mode and all pass verification; no framework file contains a product-specific term outside examples.
7. **v1.1:** every quality bar in §6 is enforced by a test, a guard or a CI step.

## 2. Users

| User | Need |
| --- | --- |
| Owner (Cecep) and his AI agent team | Start each product from a correct, verified base; port framework fixes into products. |
| Open-source developers | Build their own local-first Tauri apps with the CATerm look. |
| End users of generated apps (families, teams, small businesses, developers) | Indirect: a secure, consistent, bilingual app. |

## 3. Scope

### In scope (v1.1)

- Starter template repo `caframework` (MIT), located at `~/Project/caframework`.
- Modules (switchable in `app.toml` unless marked mandatory): core shell, multi-profile + PIN + RBAC, AI assistant, Pro package (always generated, v1.1), feedback, updater, crash reporting, **backup/restore (mandatory, v1.1)**, **data import/export (mandatory, v1.1)**, **integrations (v1.1)**, **scheduler (v1.1)**.
- Sync-ready data conventions and a change log (no sync engine).
- Generator `cargo xtask new-app` and **wizard** (`cargo xtask wizard`, App Builder page in dev builds) (v1.1).
- Sample menu "Notes".
- Platforms: Linux, Windows, macOS (desktop) and Android, from one Tauri 2 codebase with responsive UI.
- Documentation for creating an app, adding a menu, adding an entity, adding an integration, adding a job.

### Out of scope (v1.1)

- Sync engine, relay server, shared/team vault cryptography (designed later with Pro teams).
- Pro team management and multi-device sync.
- In-app payment processing (purchases happen on the GCC/Mayar.id web checkout).
- iOS.
- Product-specific features (finance, API client, networking…).
- A runtime plugin system or shared-package distribution. Template only.
- GCC server changes beyond documenting the required contract (tracked as an external dependency).

## 4. Source baseline

- Source: CATerm repo, version 2.1.12, commit `601693c` (re-pin to the latest commit at task S0 and record it).
- **Keep and generalise (Rust core):** `ai`, `audit`, `backup`, `crash`, `db`, `error`, `feedback`, `paths`, `prefs`, `pro`, `secret`, `vault`. Evaluate `sync` and `teams` (keep only what is generic, behind feature flags).
- **Remove (Rust core):** `ftp`, `groups`, `investigations`, `keys`, `local_fs`, `monitor`, `s3`, `scp`, `sftp`, `snippets`, `ssh`, `store` (host store), `tunnels`, `vfs`, `webdav`, and their commands.
- **Keep and generalise (frontend):** `PageContainer`, `PageHeader`, `CommandPalette`, `NotificationCenter`, `LockScreen`, `UpdateToast`, `AboutModal`, `FeedbackModal`, `FeedbackWidget`, `CrashReportModal`, `LanguageSwitcher`, `ProfileMenu`, `ProfileAvatar`, `AvatarPicker`, `Logo`, `AiChatPanel`, `AiSettingsForm`, `ProLoginForm`, `SponsorWall` (optional), `GridFlowBackground` (optional); stores `theme`, `appearance`, `uiNotifications`, `commandPalette`, `profile`, `pro`, `aiChat`, `feedbackStore`, `updater`; `shortcuts.ts`, `navItems.ts`, `errors.ts`, `i18n`. **v1.1:** `ProTeamPanel` is removed until Pro teams ship (decision D-4).
- **Remove (frontend):** `SessionViewport`, `TerminalPane`, `TerminalAutocomplete`, `SessionFileManager`, `RemoteFileEditor`, `HostDetailPanel`, `OsIcon`, `DirectorySync`, `VpsRecommendation`, SSH-specific parts of `WorkspaceMenu`; stores `activeSession`, `sessionTabs`, `sessionView`, `monitorStore`, `terminalPrefs`, `hostPrefs` (evaluate `workspaceStore`); `data/terminalCommands*`; routes `session`, `sftp`, `port-forwarding`, `monitoring`, `command-logs`, `investigations`, `ssh-keys`, `groups`, `snippets`; dependencies `@xterm/*`; CodeMirror becomes optional.
- `prompt-studio` is generalised into the AI Assistant module; `teams` folds into Pro (later); `contribution` becomes an optional "Support" page.

## 5. Functional requirements

### FR-1 App identity and configuration (`app.toml`)

A single file defines a product. The generator, the wizard and the build read it; after generation the code may be edited freely. **v1.1** adds `[brand]` (mandatory), `[rbac]`, `[pro]`, `[[integrations]]`, `[[jobs]]` and `[[entities]]`, and makes `backup` mandatory. `[window]` is an optional section (decision D-1).

```toml
[app]
name = "Contoh Kas"
slug = "contohkas"              # crate/package prefix, data dir name
code = "KAS"                    # error code prefix -> KAS-VAULT-001
identifier = "com.fathforce.contohkas"
accent = "emerald"              # violet sky emerald amber rose cyan indigo teal orange pink
default_locale = "id"           # id | en
locales = ["id", "en"]
platforms = ["linux", "windows", "macos", "android"]
caframework_version = "0.2.0"   # recorded by the generator

[brand]                         # v1.1, mandatory; wizard pre-fills the owner's defaults
developer = "Cecep Saeful Azhar Hidayat, ST"
company = ""                    # optional
website = "https://www.cecepazhar.com"
email = "hi@cecepazhar.com"
phone = "+6285220696117"        # E.164; shown as 0852 2069 6117 for locale id
support_url = ""                # optional; defaults to website
copyright = "© {year} Cecep Saeful Azhar Hidayat, ST"
logo = "assets/logo.svg"
show_on_lock_screen = true

[window]                        # optional
width = 1200
height = 800

[modules]
profiles = true
ai = true
pro = true                      # v1.1: Pro package is always generated; false only hides it
feedback = true
updater = true
crash = true
integrations = true
scheduler = true
support_page = false
# backup and data import/export are mandatory and cannot be listed here

[rbac]                          # v1.1 (replaces [profiles].roles / owner_role)
super_role = "super_admin"
allow_runtime_roles = true      # super role may create roles and edit the matrix at runtime

[[rbac.roles]]
key = "super_admin"
label_en = "Super admin"
label_id = "Super admin"
permissions = ["*"]

[[rbac.roles]]
key = "admin"
label_en = "Admin"
label_id = "Admin"
permissions = ["notes:*", "users:view", "backup:export", "data:export", "scheduler:view"]

[[rbac.roles]]
key = "member"
label_en = "Member"
label_id = "Anggota"
permissions = ["notes:view", "notes:create", "notes:update", "notes:delete"]

[[rbac.roles]]
key = "viewer"
label_en = "Viewer"
label_id = "Pengamat"
permissions = ["notes:view"]

[profiles]
pin_required_for = ["super_admin", "admin", "member"]
age_stages = []                 # optional, product-defined: [{ key, min_age, max_age }]

[endpoints]                     # empty = feature disabled at build time
gcc_base_url = ""
updater_manifest_url = ""

[pro]                           # v1.1
gcc_product_id = ""             # product id on GCC; empty = licence server not configured
trial_days = 14
device_limit = 3
offline_grace_days = 7

[pro.features]                  # feature key -> "free" | "pro"
ai_assistant = "pro"
scheduled_backup = "free"
data_export_xlsx = "pro"

[[integrations]]                # v1.1
key = "gcc"
kind = "gcc"                    # gcc | rest
base_url_from = "endpoints.gcc_base_url"

[[integrations]]
key = "weather"
kind = "rest"
base_url = ""                   # empty + user_configurable = set in Settings
user_configurable = true
auth = "api_key_header"         # none | api_key_header | api_key_query | bearer | basic | hmac_sha256 | oauth2_client_credentials
auth_header = "X-Api-Key"
timeout_ms = 15000
retries = 2

[[jobs]]                        # v1.1; built-in jobs are listed by the framework, products add their own
key = "backup.scheduled"
schedule = "0 2 * * 1"          # cron (5 fields, local time) or preset: hourly | daily | weekly | monthly
enabled = true

[[entities]]                    # v1.1; data import/export registry (sample shows Notes)
key = "notes"
label_en = "Notes"
label_id = "Catatan"
natural_key = []                # optional unique business key for upserts
formats = ["csv", "xlsx", "json"]

[[menus]]
key = "notes"
label_en = "Notes"
label_id = "Catatan"
icon = "M3 7h18M3 12h18M3 17h18"   # 24x24 outline SVG path
accent = ""                        # optional override
group = "daily"                    # sidebar group key (labels via i18n)
permission = "notes:view"          # v1.1: replaces roles = [...]
```

Requirements: validation of the file with readable errors (path, line, column); unknown keys rejected; menu, role, integration, job and entity keys unique and URL-safe; at least one menu; `[brand]` complete and well-formed; every permission refers to a known resource and action; the super role holds `*`.

### FR-2 Shell, navigation, and design system (identical to CATerm)

- Custom title bar with drag region and window commands (hidden on Android).
- Sidebar built from the menu list, grouped, filtered by the active profile's permissions; responsive drawer on narrow screens.
- Command Palette listing every visible menu and global commands; shortcuts from one `shortcuts.ts`.
- Notification center and toasts; confirm modal for destructive actions.
- Tokens, `PageContainer`, `PageHeader` (literal accent class map), neutral palette, dark/light themes.
- Every data page supports loading, empty, error, and backend-unavailable states.
- **v1.1:** brand footer (developer, website, email) on the lock screen when `show_on_lock_screen`; full brand block in About.

### FR-3 Settings registry

Settings is one page with sections contributed by modules (a typed registry), so a product adds its own section without editing the framework page. Framework sections: Appearance, Language, Profiles & Security, **Roles & Permissions (v1.1)**, Backup & Restore, **Data Import/Export (v1.1)**, **Integrations (v1.1)**, **Schedules (v1.1)**, AI Assistant, Updates, Privacy (crash reports), Shortcuts, Pro, About. Sections appear only when the active profile has the matching `view` permission.

### FR-4 Vault and storage

- Encrypted SQLite (SQLCipher) keyed from a master password via Argon2id; lock/unlock; change master password; reset with explicit confirmation.
- **Recovery code** (24 words, generated at setup, must be confirmed as saved): the data key is wrapped twice, by the master-password key and by the recovery-code key, so a forgotten password can be recovered; the super role can regenerate the code, which revokes the old one. The code is never stored in plaintext.
- Schema migrations with a version table and a migration runner; migration tests.
- Plain-header guard test (the on-disk file never starts with `SQLite format 3`).

### FR-5 Sync-ready data conventions (no sync engine yet)

Every user-data table has: `id` (UUIDv7 text), `created_at`, `updated_at` (unix ms), `deleted_at` (nullable tombstone), `rev` (integer, increments on change), `origin_device_id`, and, when profiles are enabled, `owner_profile_id` and `visibility`. Every write appends to `change_log(entity, entity_id, op, rev, at, profile_id, device_id)`. Deletes are soft by default. A test fails if any table violates the convention.

### FR-6 Multi-profile, PIN, RBAC, ownership

- Two lock levels: **vault lock** (master password, protects data at rest) and **profile lock** (PIN, separates people sharing a device). Auto-lock timers for both, configurable.
- Profile picker after vault unlock; profiles: display name, avatar, role, optional birth year (for products that need age stages), PIN hash (Argon2id, stored inside the vault).
- A minimal **profile index** (display name + avatar only) may be stored outside the vault so the lock screen can greet users; a setting hides names on the lock screen.
- **RBAC (v1.1).** Permissions are `<resource>:<action>` with actions `view`, `create`, `update`, `delete`, `export`, `import`, `manage`, `run`; wildcards `<resource>:*` and `*`. Resources are the menus, entities, integrations, jobs and system areas (`users`, `roles`, `settings`, `backup`, `data`, `integrations`, `scheduler`, `ai`, `pro`, `audit`). The resource catalogue is fixed at build time from `app.toml`; roles and their permission sets start from `app.toml` and, when `allow_runtime_roles`, the super role can create, edit and delete roles at runtime. The super role always holds `*`; at least one profile always has the super role. Every check runs in the **core**, never only in the UI.
- Ownership and visibility primitives: every record has an owner profile and one of `shared`, `private_summary` (others see only aggregates), `private` (owner only). Core query helpers apply the filter.
- **Super role exception (v1.1, decision D-2 revised).** Profiles with the super role can read every row, including other profiles' `private` rows. Each such read is recorded in the audit log (`PRIVATE_READ`, profile, entity, count), and the UI marks those rows with the owner's name and a "private" badge. Tests prove that no other role can read another profile's `private` rows through any command.
- Honest limitation documented in the UI and docs: on one shared device, profile separation is enforced by the app, not by separate encryption keys; the super role can read private data.

### FR-7 AI assistant module

- Modes: Off, Bring-your-own key (any OpenAI-compatible endpoint, including local Ollama), Hosted (via CA Pro quota, only if Pro is configured).
- User-configurable: base URL, model, temperature, response language, tone, custom instructions. API keys stored in the vault, never in `localStorage`.
- **Context providers:** each product registers functions that build the context sent to the model. Default privacy level is **anonymised summary** (aggregates, no names); detailed data only with per-conversation consent; local endpoints may be allowed full detail by a setting. Providers read through the active profile's permissions and visibility.
- Product **guardrails**: a fixed, non-user-editable system-prompt section per product, composed before user instructions.
- Hosted-mode errors are shown to the user; BYO errors are shown inline in the chat panel without a modal.

### FR-8 Pro package, feedback, updater, crash reporting

- **Pro package (v1.1).** Every generated app contains the Pro module: plans page (plans and prices fetched from GCC), licence activation, trial, device limit, offline grace period, licence status in About and Settings. Developers mark features `free` or `pro` in `[pro.features]`; the core enforces entitlements (`ProError::FeatureLocked`) and the UI shows an upgrade prompt. Licences are signed by GCC and verified offline with a public key embedded at build time; the licence token lives in the vault. Purchases open the GCC/Mayar.id checkout in the browser. Without a configured GCC product, Pro features stay locked with an explanatory message; debug builds may unlock them with an explicit developer switch. `app_id` is sent on every call.
- Feedback through the GCC proxy; no secrets in the client.
- Updater with minisign public key per app and `latest.json` from CI.
- Crash reporting: panic hook, local dump `0600`, PII scrubber, opt-in preview, "don't ask again".

### FR-9 Backup and restore (mandatory, v1.1)

Backup cannot be disabled. Encrypted backup file with format version; restore with integrity check; round-trip test (create → backup → wipe → restore → identical data). **Scheduled backup** (configurable interval, default weekly) to a user-chosen folder, with an overdue reminder (default 30 days) and a visible "last successful backup" time. From v0.2.0 the schedule runs on the scheduler (FR-16).

### FR-10 i18n and formatting

Typed dictionaries (`en.ts` reference, `id.ts` typed against it); locale-aware number, currency, and date formatting helpers; Hijri date support is product-level (not in the framework).

### FR-11 Sample menu "Notes"

A complete vertical slice (title, body, tags; owner and visibility; import/export) showing the pattern: core module → migration → commands → API binding → page → i18n → tests. The generator includes it only with `--with-sample`.

### FR-12 Generator `cargo xtask new-app`

- Inputs: `--config app.toml`, `--out <dir>`, optional `--with-sample`, `--dry-run`.
- Copies the template, renames crates/packages/identifiers/product strings/data-dir/storage keys/error prefix, applies accent, brand and logo placeholders, generates `navItems.ts`, i18n menu keys, one placeholder route per menu, Settings registry entries, permission catalogue, integration, job and entity registrations, CI names, a fresh `Notes/prd.md` and `Notes/task.md` stub, and records `caframework_version`.
- Generates a new minisign key pair outside the repo and writes only the public key.
- Refuses to overwrite a non-empty directory.
- End-to-end test: generate into a temp dir and run the verification commands on it.

### FR-13 Android

The same app builds an APK; title bar hidden; drawer navigation; touch targets ≥ 44 px; signing documented (keystore outside the repo).

### FR-14 Open-source hygiene

MIT licence, `CONTRIBUTING.md`, `SECURITY.md`, no secrets or private endpoints in the repo, CI guards kept from CATerm (no tracked binaries/DBs, no files over 1 MB, no absolute dev-machine paths).

### FR-15 Integrations and GCC adapter (v1.1)

- One connector layer in the core. Each integration is declared in `[[integrations]]` with base URL (fixed or user-configurable), authentication (`none`, `api_key_header`, `api_key_query`, `bearer`, `basic`, `hmac_sha256`, `oauth2_client_credentials`), timeout, retry policy and allowed hosts.
- Secrets live in the vault and are set in Settings → Integrations by profiles with `integrations:manage`; the frontend never receives them.
- Features: connection test, request logging with redaction, retries with exponential back-off for idempotent requests, per-integration rate limit, offline outbox for writes (retried by the scheduler, idempotency key per request).
- GCC is a built-in adapter covering licensing, Pro plans, hosted AI, feedback, crash reports, update manifest and a generic data push/pull contract, documented in `docs/backend-contract.md`.
- Products add integrations through `app.toml` and one Rust file per adapter, without editing framework code.

### FR-16 Scheduler (v1.1)

- Modules and products register jobs (key, title, default schedule, required permission, whether the vault must be unlocked, catch-up policy, maximum runtime).
- Schedules: 5-field cron in local time or presets (hourly, daily, weekly, monthly); each job can be enabled, disabled, edited and run now from Settings → Schedules with a human-readable preview ("Setiap Senin 02:00").
- Run history with status and log per run (retention configurable, default 500 runs); one instance per job at a time.
- Desktop: optional system tray mode (close to tray) and optional start at login, both off by default. Missed runs while the app was closed are caught up on next start according to the job's policy. Android: jobs run while the app is in use, plus catch-up; platform limits documented.
- Built-in jobs: scheduled backup, backup overdue reminder, integration outbox flush, Pro licence refresh.

### FR-17 Data import and export (mandatory, v1.1)

- Every entity registered in `[[entities]]` gets import and export in CSV, XLSX and JSON (formats selectable per entity).
- Import flow: choose file → automatic column mapping (editable) → preview with per-row validation → mode (insert only, upsert by id or natural key) → dry-run summary → execute in one transaction (all-or-nothing, or skip invalid rows when the user chooses) → report with a downloadable file of failed rows and reasons. Imported rows go through the sync helper and visibility rules; the importer becomes the owner.
- Export respects permissions and visibility; spreadsheet exports neutralise formula injection.
- Downloadable import templates per entity; documented limits (default 50 000 rows or 20 MB per file).

### FR-18 Developer branding (v1.1)

`[brand]` is mandatory. The wizard pre-fills the owner's details (developer Cecep Saeful Azhar Hidayat, ST; website https://www.cecepazhar.com; email hi@cecepazhar.com; phone 0852 2069 6117) from `~/.config/caframework/brand.toml`. Brand data appears in About, the lock-screen footer (configurable), installer metadata (publisher, copyright, homepage, maintainer email), `README.md`, `NOTICE`, `SECURITY.md` contact, crash/feedback footers and the HTTP User-Agent. Other developers using the MIT framework may replace the values.

### FR-19 Wizard and App Builder (v1.1)

- `cargo xtask wizard` asks step by step: identity (name → suggested slug, code, identifier), brand (pre-filled), accent with preview, locales, platforms, modules, roles and permission presets, menus (key, labels, icon from a bundled icon set, group, permission), entities, integrations, jobs, Pro features; shows a review of the resulting `app.toml`; validates; writes it; optionally runs `new-app`.
- Non-interactive mode `--answers answers.toml` for agents and CI.
- App Builder page in CAFramework's own dev build offers the same steps as a form with live preview (sidebar, accent, roles matrix) and generates the app through the same library.
- One schema drives the CLI, the GUI and validation (JSON Schema generated from the Rust types).

## 6. Non-functional requirements

| Area | Requirement |
| --- | --- |
| Security | Keys in `secrecy`/`zeroize`; no secrets in logs, errors, crash dumps; capability and permission checks in core; `cargo deny` clean. |
| Privacy | Zero telemetry by default; crash/feedback opt-in; AI default = anonymised summary; super-role reads of private rows are audited. |
| **Stability (v1.1)** | No `unwrap`/`expect`/`panic!`/indexing panics in non-test core code (clippy `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` denied); every parser of external input (`app.toml`, backup files, import files, HTTP responses, licence tokens) has property or fuzz tests (≥ 10 000 cases in CI); all file writes atomic; DB `PRAGMA integrity_check` on unlock with a clear error; migrations tested from every previous framework version; scheduler jobs isolated (a failing job never crashes the app); a 30-minute soak test (scheduler, auto-lock, imports) without errors or memory growth over 10 %. |
| **Maintainability (v1.1)** | Layered architecture enforced by arch tests (core has no GUI deps; commands ≤ 15 lines; frontend calls only generated bindings); registries (menus, settings, entities, integrations, jobs, permissions, AI context providers) let products extend without editing framework files; source files ≤ 600 lines and functions ≤ 80 lines (guarded, exceptions listed); core line coverage ≥ 80 % and frontend ≥ 70 % (`cargo llvm-cov`, `vitest --coverage`), never decreasing; every public core item documented; ADRs for design decisions; semver tags and `CHANGELOG.md` with port notes; pinned toolchain. |
| **Scalability (v1.1)** | Every list command is paginated (cursor, max 500 per page) and guarded by an arch test; indexes on `owner_profile_id`, `visibility`, `updated_at`, `deleted_at` for every sync table; reference dataset of 100 000 rows per entity: list page ≤ 100 ms, search ≤ 300 ms, import of 50 000 rows ≤ 30 s, export ≤ 15 s, backup ≤ 60 s on the owner's laptop; blocking work off the UI thread (`run_blocking`); connector concurrency limits; idle memory ≤ 250 MB; benchmarks run in CI with a regression threshold of 20 %. |
| Performance | Cold start of the empty generated app ≤ 2 s on the owner's ThinkPad X1 Yoga Gen 3 (measured, not estimated); release profile `opt-level = "z"`, `lto = "fat"`. |
| Quality | `cargo test`, `clippy -D warnings`, `fmt --check`, `deny`, `npm run check`, `npm run build` all green; arch test and error-code golden test kept. |
| Accessibility | Visible focus, labelled icon buttons, WCAG AA contrast in both themes. |
| Genericity (v1.1) | No product-specific term (family, child, cash, household…) in framework code, defaults or UI strings outside examples and the Notes sample; guarded by a word list. |

## 7. Key decisions

| # | Decision |
| --- | --- |
| D1 | Form: starter template (not shared library); extraction into packages may come later. |
| D2 | Modules: core, multi-profile + PIN, AI assistant, Pro licensing + feedback — switchable (backup and data import/export mandatory from v1.1). |
| D3 | Sync: data conventions and change log now; engine later with Pro. |
| D4 | Generator: `app.toml` + `cargo xtask new-app`. |
| D5 | Open-source MIT; secrets and licence enforcement stay server-side. |
| D6 | Sample menu "Notes" included. |
| D7 | Platforms: Android + desktop from one Tauri 2 codebase. |
| D8 | Bilingual ID/EN from day one. |
| D9 | Vault recovery code (24 words) and scheduled backup are framework features. |
| D10 (v1.1) | Framework is product-agnostic; product terms appear only in examples. |
| D11 (v1.1) | RBAC with a permission matrix; roles from `app.toml`, editable at runtime by the super role. |
| D12 (v1.1) | The super role can read all data, including private rows; every such read is audited. |
| D13 (v1.1) | Integrations through a connector registry; GCC is the built-in adapter. |
| D14 (v1.1) | Scheduler in-app with optional tray and start-at-login, plus catch-up. |
| D15 (v1.1) | Backup and per-entity import/export (CSV, XLSX, JSON) are mandatory. |
| D16 (v1.1) | `[brand]` is mandatory; the owner's details are the wizard defaults. |
| D17 (v1.1) | Pro package always generated, with Free/Pro feature gating; payments on GCC/Mayar.id; Pro teams and sync deferred. |
| D18 (v1.1) | Wizard as CLI (`cargo xtask wizard`) and App Builder page, both on one schema. |
| D19 (v1.1) | Release split: v0.1.0 = audit remediation + v1.1 foundations (brand, RBAC schema and core checks, mandatory backup, super-role rule); v0.2.0 = remaining v1.1 features. |
| D20 (v1.1) | Stability, maintainability and scalability are measurable requirements (§6), enforced by tests, guards and CI. |

## 8. Dependencies and risks

| Item | Type | Mitigation |
| --- | --- | --- |
| GCC must accept `app_id` and support licences, plans, hosted AI, feedback, crash and data endpoints | External dependency | Documented contract; mock server in tests; features disabled until the server supports them. |
| Tauri app crate cannot compile in headless cloud environments | Verification risk | Owner runs `cargo check -p <slug>-app` locally; tasks marked NOT VERIFIED until then. |
| Profile separation on a shared device is app-level only; the super role can read private data | Security limitation | Stated in UI/docs; audited reads; real per-person crypto comes with Pro multi-device. |
| Scheduled jobs do not run while the app is closed (and are limited on Android) | Platform | Tray mode, start at login, catch-up on start; documented. |
| Runtime role editing can lock users out | Usability/security | Super role always keeps `*`; at least one super-role profile enforced; audit log. |
| Stripping SSH code breaks hidden references | Regression | Grep before delete; build and tests after each removal batch. |
| Template drift between framework and products | Maintenance | Semver + changelog port notes; `caframework_version` in each app. |
| Android plugin gaps in Tauri 2 | Platform | Fallback documented per feature; Android verified in its own stage. |

## 9. Definition of done

**v0.1.0:** all `TASK_FIX.md` stages passed their gates with evidence; read-only QA audit has zero critical/high findings and zero UI-only items; tag `v0.1.0` released; a skeleton generated from it builds and launches on Linux and Android.

**v0.2.0:** all `TASK_FEATURES.md` stages passed their gates with evidence; every §6 quality bar enforced and green; three structurally different apps generated through the wizard's non-interactive mode pass verification (success criterion 6); read-only QA re-audit clean; tag `v0.2.0` released.
