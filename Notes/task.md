# CAFramework — Task List (v1.0)

> Executed with `prompt-dev.md`; requirements in `prd.md` (FR-n references below).
> Tick `[x]` only when the proof is recorded in `Notes/evidence/S<n>.md` of the `caframework` repo. `[~]` = partial/UI only (explain), `[!]` = blocked.
> One stage at a time. At each gate: write evidence, report to the owner **in Bahasa Indonesia**, and stop for approval.

## Evidence format

```
### <TASK-ID> <title>
Command: `<exact command>`
Output:
<real output, trimmed but unedited>
Verdict: VERIFIED | UI-ONLY | NOT VERIFIED (reason)
```

## Standard verification set ("STD")

| Check | Command | Pass |
| --- | --- | --- |
| Rust tests | `cargo test --workspace` | 0 failed |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings |
| Format | `cargo fmt --all -- --check` | no diff |
| Deps | `cargo deny check` | ok |
| Arch guard | `cargo test -p caf-core --test arch` | pass |
| Error registry | `cargo test -p caf-core --test error_codes_unique` | pass, no golden diff |
| Frontend | `cd frontend && npm ci && npm run check && npm run build` | 0 errors, build ok |
| App crate (owner's machine) | `cargo check -p caf-app` | no error |

If a check cannot run in the agent's environment, mark it NOT VERIFIED and give the owner the exact command.

---

## S0 — Intake and source pin

- [ ] **S0.1** Read `prd.md`, `prompt-dev.md`, this file. Report understanding in ≤ 10 lines and any questions (numbered, each with a recommended default). Proof: report.
- [ ] **S0.2** Pin the CATerm source: record commit hash and version. Proof: `git -C ~/Project/caterm log -1 --format='%H %s'` and `grep '^version' Cargo.toml`.
- [ ] **S0.3** Inventory every CATerm module, component, store, route, command, and dependency, classified KEEP / GENERALISE / REMOVE / EVALUATE (start from `prd.md` §4). Proof: `Notes/inventory.md` with counts that match `grep` output (e.g. number of `#[tauri::command]`).
- [ ] **S0.4** Record the CATerm baseline: run STD on CATerm unchanged. Proof: outputs (so later regressions are distinguishable from pre-existing failures).

**Gate S0:** owner approves the inventory and answers to questions.

## S1 — Fork, rename, green baseline

- [ ] **S1.1** Create `~/Project/caframework` as a fresh git repo from the pinned CATerm tree (no CATerm history needed; keep attribution in `NOTICE`). Proof: `git log --oneline | head -3`, `NOTICE` content.
- [ ] **S1.2** Rename workspace crates to `caf-core`, `caf-cli`, `caf-app`; frontend package `caf-frontend`; product name `CAFramework`; identifier `com.fathforce.caframework`; error prefix `CAF-`; storage keys `caf-theme`, `caf_locale`, `caf_profile_v1`. Proof: `grep -rIl -i 'caterm' --exclude-dir={target,node_modules,.git} .` lists only `NOTICE`/credits.
- [ ] **S1.3** Version `0.1.0-dev` in `Cargo.toml`, `package.json`, `tauri.conf.json`. Proof: grep output.
- [ ] **S1.4** STD passes (same failures as S0.4 at most, each explained). Proof: outputs.

**Gate S1:** renamed repo builds; no unexplained new failures.

## S2 — Remove SSH-specific code

Work in small batches; after each batch run `cargo test --workspace` and `npm run check`.

- [ ] **S2.1** Remove Rust modules `ssh`, `sftp`, `scp`, `ftp`, `tunnels`, `vfs`, `s3`, `webdav`, `local_fs`, `monitor`, `keys`, `groups`, `investigations`, `snippets`, host `store`, and their commands/handler entries. Grep references first. Proof: per-batch test output; `grep -c 'tauri::command' crates/caf-app/src/commands.rs` before/after.
- [ ] **S2.2** Remove crates/deps used only by removed modules (`cargo machete` or manual check). Proof: `Cargo.toml` diff, `cargo deny check`.
- [ ] **S2.3** Remove frontend routes `session`, `sftp`, `port-forwarding`, `monitoring`, `command-logs`, `investigations`, `ssh-keys`, `groups`, `snippets`; components and stores listed in `prd.md` §4; `@xterm/*` deps. Proof: `npm run check` 0 errors; `grep -rI xterm frontend/src` empty.
- [ ] **S2.4** Clean `+layout.svelte` of session tabs, terminal viewport, monitoring, workspace SSH logic while keeping title bar, sidebar, palette, notifications, lock, crash, feedback, updater. Proof: STD; screenshot of the empty shell on the owner's machine.
- [ ] **S2.5** Remove SSH strings from `en.ts`/`id.ts`. Proof: `npm run check`; grep for `ssh|sftp|terminal|tunnel` in locales returns only intentional hits (listed).

**Gate S2:** STD green; app launches as an empty CATerm-style shell; zero SSH code/deps.

## S3 — `app.toml` and identity

- [ ] **S3.1** Define the `app.toml` schema (prd FR-1) as Rust types in an `xtask` crate with strict validation (unknown keys rejected, unique URL-safe menu keys, valid accent, ≥ 1 menu). Proof: unit tests for valid file and each invalid case.
- [ ] **S3.2** Add the framework's own `app.toml` (CAFramework demo identity, one "Notes" menu). Proof: file.
- [ ] **S3.3** Build-time codegen from `app.toml`: `frontend/src/lib/generated/app.ts` (name, slug, accent, modules, roles) and `navItems.generated.ts`; error prefix constant for Rust. Generated files carry a "generated" header and are reproducible. Proof: run codegen twice → `git diff --exit-code`.
- [ ] **S3.4** Logo/icon placeholder pipeline (single source SVG → Tauri icon set via `tauri icon`). Proof: generated icon files list.

**Gate S3:** changing `name`, `accent`, or menus in `app.toml` and re-running codegen changes the app accordingly (screenshots before/after).

## S4 — Shell, navigation, design system, Settings registry

- [ ] **S4.1** `app.css` tokens and utilities identical to CATerm (diff shows only intentional changes). Proof: diff.
- [ ] **S4.2** `PageContainer`, `PageHeader` (literal 10-accent map), `CommandPalette`, `NotificationCenter`, `UpdateToast`, `AboutModal`, `LanguageSwitcher`, `ProfileMenu`, `Logo` generic (no product strings). Proof: grep shows no hard-coded product name; `npm run check`.
- [ ] **S4.3** Sidebar from generated nav items with groups and role filtering (role filter is a no-op when profiles are disabled). Proof: unit test of the filter; screenshots.
- [ ] **S4.4** Responsive drawer below `md`; title bar hidden on Android (compile-time and runtime check). Proof: narrow-width screenshot; cfg gate in code.
- [ ] **S4.5** Settings registry: typed section registration (`id`, label key, icon, order, component, `requires` module); framework sections from prd FR-3. Proof: a test section added from outside the framework page appears without editing `settings/+page.svelte` (show diff).
- [ ] **S4.6** Shortcuts in one list; palette shows menus + global commands. Proof: manual test list.
- [ ] **S4.7** Four-state components (`LoadingState`, `EmptyState`, `ErrorState`, `BackendUnavailable`) and confirm modal. Proof: Storybook-like demo route `/_dev/states` (dev builds only) screenshots, both themes.
- [ ] **S4.8** Locale-aware formatting helpers (number, currency, date) with tests for `id` and `en`. Proof: test output.

**Gate S4:** owner approves screenshots (desktop/narrow, dark/light).

## S5 — Core storage, errors, sync-ready conventions

- [ ] **S5.1** Error taxonomy with prefix from `app.toml` (`CAF-<DOMAIN>-NNN`), serialisation `{ code, message, domain }`, golden registry regenerated. Proof: golden test, `docs/error-codes.md` diff.
- [ ] **S5.2** Migration runner with `schema_version` table; framework migrations numbered; product migrations in their own range. Proof: test upgrading from an empty DB and from the previous version.
- [ ] **S5.3** Sync-ready columns helper and `change_log` table (prd FR-5); every write path appends to the log in the same transaction. Proof: tests for insert/update/soft-delete producing correct `rev` and log rows; rollback leaves no log row.
- [ ] **S5.4** Convention guard test: introspect `sqlite_master` after all migrations and fail if any user-data table lacks required columns. Proof: test passes; deliberately broken temp migration makes it fail (show, then revert).
- [ ] **S5.5** UUIDv7 ids (`uuid` crate with `v7`), device id generated once and stored. Proof: tests.
- [ ] **S5.6** Plain-header guard test kept. Proof: test output.
- [ ] **S5.7** Backup/restore with format version and integrity check (prd FR-9). Proof: round-trip test.
- [ ] **S5.7b** Scheduled backup to a chosen folder (interval setting, overdue reminder, last-success time). Proof: tests with injected clock; backup file listing.
- [ ] **S5.8** Crash module kept: panic hook first, `0600`, scrubber tests, three commands, modal. Proof: tests + forced panic screenshot.

**Gate S5:** STD green; guard tests demonstrably able to fail.

## S6 — Vault, multi-profile, PIN, roles, ownership

- [ ] **S6.1** Vault lock (master password) kept and renamed; change/reset flows. Proof: tests for wrong password, change, reset confirmation.
- [ ] **S6.1b** Recovery code: 24-word generation, confirmation step, DEK wrapped by both password key and recovery key, unlock with recovery code then set new password, regenerate-and-revoke. Proof: tests (old code fails after regeneration; plaintext code absent from disk).
- [ ] **S6.2** Profiles table (sync-ready) + profile index outside the vault (display name + avatar only) + setting to hide names on lock screen. Proof: tests; inspect index file shows no sensitive fields.
- [ ] **S6.3** PIN: Argon2id hash in vault, attempt limit with back-off, owner can reset a member's PIN, roles in `pin_required_for` must set one. Proof: tests for set/verify/lockout/reset.
- [ ] **S6.4** Two auto-lock timers (profile → picker, vault → master password). Proof: tests with injected clock; manual check.
- [ ] **S6.5** Roles and capabilities from `app.toml`; checks in core command layer (owner-only actions: manage profiles, reset, change master password). Proof: tests calling commands as a non-owner get `CAF-AUTH-*` errors.
- [ ] **S6.6** Ownership + visibility filter helpers (`shared` / `private_summary` / `private`); aggregates for `private_summary`. Proof: tests proving a second profile cannot read `private` rows or `private_summary` details through any query helper.
- [ ] **S6.7** UI: lock screen → profile picker → PIN pad (touch-friendly), profile management in Settings, active profile in `ProfileMenu`, "switch profile" in palette. Proof: screenshots + wiring proof (create profile → restart → picker shows it → PIN works).
- [ ] **S6.8** Disabled mode: with `modules.profiles = false` the app behaves like CATerm (single local profile). Proof: build with the flag off + screenshot.
- [ ] **S6.9** Document the shared-device limitation in Settings help text and `docs/security.md`. Proof: files.

**Gate S6:** all auth/permission tests green; owner tests the flow on his machine.

## S7 — AI assistant module

- [ ] **S7.1** Provider abstraction: Off / BYO OpenAI-compatible (incl. Ollama) / Hosted; config (base URL, model, temperature, language, tone, custom instructions); keys in vault. Proof: tests with a mock HTTP server; grep shows no key in `localStorage` code.
- [ ] **S7.2** Context-provider registry with privacy levels (anonymised summary default; detailed with per-conversation consent; full detail allowed for local endpoints by setting). Proof: tests asserting the outgoing payload for each level (no names in summary level).
- [ ] **S7.3** Product guardrail section (fixed, non-editable) composed before user instructions. Proof: test of prompt composition order.
- [ ] **S7.4** UI: `AiChatPanel` + AI settings section; consent prompt; error states (`AI_QUOTA_EXCEEDED`, `AI_NOT_CONFIGURED`, network). Proof: screenshots; a real call to a local Ollama or BYO endpoint on the owner's machine.
- [ ] **S7.5** Hosted mode only when Pro is enabled and an endpoint is configured; otherwise hidden. Proof: build with/without.

**Gate S7:** real completion demonstrated via BYO/Ollama; payload tests green.

## S8 — Pro licensing, feedback, updater (configurable)

- [ ] **S8.1** Pro client parameterised by `app_id`/`slug` and `endpoints.gcc_base_url`; disabled cleanly when unset. Proof: tests; build with empty endpoint shows no Pro UI.
- [ ] **S8.2** Feedback via GCC proxy; disabled when unset. Proof: tests; real request when endpoint available (else NOT VERIFIED with reason).
- [ ] **S8.3** Updater: per-app minisign public key, manifest URL from config, `UpdateToast`. Proof: update check against a local test manifest.
- [ ] **S8.4** Document the GCC `app_id` contract in `docs/backend-contract.md` (endpoints, headers, error codes). Proof: file.

**Gate S8:** all three features work when configured and leave no dead UI when not.

## S9 — Sample menu "Notes" (full vertical slice)

- [ ] **S9.1** Migration + core module (title, body, tags, owner, visibility) using sync-ready helpers. Proof: core tests.
- [ ] **S9.2** Commands (thin, `run_blocking`) + API binding mirroring Rust types. Proof: `cargo check -p caf-app` (owner), `npm run check`.
- [ ] **S9.3** Page with `PageHeader`/`PageContainer`, four states, visibility selector, import/export JSON via dialog/fs plugins, i18n. Proof: screenshots both themes.
- [ ] **S9.4** Wiring proof: create note as profile A (private) → switch to profile B (not visible) → restart → profile A still sees it. Proof: transcript + screenshots.
- [ ] **S9.5** `docs/adding-a-menu.md` written from this slice (14-step checklist). Proof: file.

**Gate S9:** S9.4 VERIFIED.

## S10 — Generator `cargo xtask new-app`

- [ ] **S10.1** Copy + rename engine (crates, packages, identifiers, product strings, data dir, storage keys, error prefix) with an explicit replacement table. Proof: unit tests on a fixture tree.
- [ ] **S10.2** Generate nav, i18n menu keys, placeholder routes (with `PageHeader` + `EmptyState`, marked UI-only), settings entries, CI names, `Notes/prd.md` + `Notes/task.md` stubs, `caframework_version`. Proof: generated tree listing.
- [ ] **S10.3** `--with-sample` toggles the Notes slice; without it, all Notes code, migrations, strings are absent. Proof: grep on generated output.
- [ ] **S10.4** New minisign key pair written outside the repo; only the public key in config. Proof: config diff; key path.
- [ ] **S10.5** Safety: refuses non-empty `--out`; dry-run flag prints the plan. Proof: test output.
- [ ] **S10.6** E2E test: generate `demoapp` (2 menus) into a temp dir, then run STD inside it. Proof: full output.
- [ ] **S10.7** `docs/creating-an-app.md`. Proof: file.

**Gate S10:** a generated app passes STD without manual edits.

## S11 — Android

- [ ] **S11.1** Android build of the framework app (Tauri 2 mobile), using the CATerm Docker/SDK approach. Proof: build log, APK path, `sha256sum`.
- [ ] **S11.2** Signing doc and keystore outside the repo. Proof: `apksigner verify --print-certs` output.
- [ ] **S11.3** On-device check: lock, profile picker, PIN pad, drawer, Notes slice. Proof: owner screenshots.
- [ ] **S11.4** List any plugin/feature gaps on Android with fallbacks. Proof: `docs/android.md`.

**Gate S11:** APK installs and passes S11.3 on a real device.

## S12 — CI, release, open-source hygiene

- [ ] **S12.1** Workflows `ci.yml`, `release.yml`, `build-macos.yml` renamed and parameterised; CATerm guards kept (no tracked binaries/DBs, no file > 1 MB, no absolute dev paths). Proof: green CI run link/log.
- [ ] **S12.2** Generator E2E runs in CI. Proof: CI log.
- [ ] **S12.3** `LICENSE` (MIT), `NOTICE`, `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md` with "port notes" section. Proof: files.
- [ ] **S12.4** Secret scan of the whole history (e.g. `gitleaks detect`). Proof: output with 0 findings.

**Gate S12:** CI green on a clean clone.

## S13 — Documentation

- [ ] **S13.1** `docs/architecture.md` (layers, modules, data conventions, lock levels, AI privacy levels). Proof: file.
- [ ] **S13.2** `docs/security.md`, `docs/backend-contract.md`, `docs/error-codes.md` (generated), `docs/android.md`, `docs/creating-an-app.md`, `docs/adding-a-menu.md`, `docs/porting-framework-fixes.md`. Proof: files; links checked.

**Gate S13:** owner can create an app by following the docs alone (owner confirms).

## S14 — Independent QA audit (read-only) and fix loop

- [ ] **S14.1** Run the audit prompt (Appendix A of `prompt-dev.md`) in a separate session that does not edit code. Proof: `Notes/audit/<date>.md`.
- [ ] **S14.2** Fix findings one per commit with regression tests. Proof: commit list + test output.
- [ ] **S14.3** Re-audit until zero critical/high and zero UI-only items. Proof: final audit file.

**Gate S14:** clean audit.

## S15 — Release v0.1.0 and hand-off to CACash

- [ ] **S15.1** STD on a clean clone; tag `v0.1.0`; release notes. Proof: outputs, tag.
- [ ] **S15.2** Generate the CACash skeleton from the CACash `app.toml` (supplied by the owner after CACash brainstorming) into its own repo; STD passes; launches on Linux and Android. Proof: outputs + screenshots.
- [ ] **S15.3** Record framework backlog discovered during the work (candidates for later extraction into packages). Proof: `Notes/backlog.md`.

**Gate S15:** owner accepts v0.1.0 and the CACash skeleton.
