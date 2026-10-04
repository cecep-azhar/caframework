# Fix ledger — CAFramework remediation
Base commit: 7ec9e64 (HEAD: e41aed3) · Branches: main (direct-to-main mode per owner instruction) · Started: 4 October 2026 · Runner: x1-bench
Current plan: TASK_FIX.md

## Status
| Stage | Status | Gate | Evidence | Handoff |
|---|---|---|---|---|
| F0 | DONE | GREEN | Notes/evidence/fix/F0.md | Scaffolding and baseline verified, logs recorded (commit `6c95d1a`) |
| F1 | DONE | GREEN | Notes/evidence/fix/F1.md | Toolchain fixes done and CI rewritten |
| F2 | DONE | GREEN | Notes/evidence/fix/F2.md | Leftovers & artifacts cleanup complete |
| F3 | DONE | GREEN | Notes/evidence/fix/F3.md | - |
| F4 | DONE | GREEN | Notes/evidence/fix/F4.md | - |
| F5 | DONE | GREEN | Notes/evidence/fix/F5.md | - |
| F6 | DONE | GREEN | Notes/evidence/fix/F6.md | 2026-10-04 |
| F7 | DONE | GREEN | Notes/evidence/fix/F7.md | 2026-10-04 |
| F8 | DONE | GREEN | Notes/evidence/fix/F8.md | 2026-10-04 |
| F9 | DONE | GREEN | Notes/evidence/fix/F9.md | 2026-10-04 |
| F10 | DONE | GREEN | Notes/evidence/fix/F10.md | 2026-10-04 |
| F11 | DONE | GREEN | Notes/evidence/fix/F11.md | 2026-10-04 |
| F12 | IN_PROGRESS | - | Notes/evidence/fix/F12.md | - |
| F13 | TODO | - | Notes/evidence/fix/F13.md | - |
| F14 | TODO | - | Notes/evidence/fix/F14.md | - |
| F15 | TODO | - | Notes/evidence/fix/F15.md | - |
| F16 | TODO | - | Notes/evidence/fix/F16.md | - |
| F17 | TODO | - | Notes/evidence/fix/F17.md | - |
| F18 | TODO | - | Notes/evidence/fix/F18.md | - |
| F19 | TODO | - | Notes/evidence/fix/F19.md | - |
| F20 | TODO | - | Notes/evidence/fix/F20.md | - |

## Tasks
| Task | Status | Evidence anchor | Note |
|---|---|---|---|
| F0.1 | DONE | F0.md#f01-work-branch-and-pin | Branch pinned on main, toolchain recorded |
| F0.2 | DONE | F0.md#f02-notes-scaffolding | Notes files and amendments initialized |
| F0.3 | DONE | F0.md#f03-run-the-audit-verification-checklist | Audit §5 verification in throwaway clone |
| F0.4 | DONE | F0.md#f04-baseline-measurements | Ratchets baseline measurement |
| F0.5 | DONE | F0.md#f05-classify-commit-message-claims | T-9 commit claims classification |
| F1.1 | DONE | F1.md#f11 | Clippy unwrap_used fixed via LazyLock in ai.rs & main.rs |
| F1.2 | DONE | F1.md#f12 | Format and cargo-deny policy green (yoke-derive updated, deny.toml configured) |
| F1.3 | DONE | F1.md#f13 | caf-xtask guard subcommand implementation |
| F1.4 | DONE | F1.md#f14 | Hard-coded strings and sync writes scanner |
| F1.5 | DONE | F1.md#f15 | CI workflow rewrite |
| F1.6 | DONE | F1.md#f16 | Vitest test runner |
| F1.7 | DONE | F1.md#f17 | Quality-bar tooling and lints |
| F2.1 | DONE | F2.md#f21 | Stale permissions cleanup |
| F2.2 | DONE | F2.md#f22 | Frontend leftovers cleanup |
| F2.3 | DONE | F2.md#f23 | Rust leftovers cleanup |
| F2.4 | DONE | F2.md#f24 | Packaging and root files cleanup |
| F2.5 | DONE | F2.md#f25 | Absolute paths in docs |
| F2.6 | DONE | F2.md#f26 | Untracked CATerm artefacts |
| F2.7 | DONE | F2.md#f27 | Genericity guard |
| F3.1 | IN_PROGRESS | F3.md#f31 | Schema types v1.1 |
| F3.2 | TODO | F3.md#f32 | Validation with readable errors |
| F3.3 | TODO | F3.md#f33 | Framework app.toml |
| F3.4 | TODO | F3.md#f34 | Codegen outputs and --check |
| F3.5 | TODO | F3.md#f35 | Icon pipeline |
| F3.6 | TODO | F3.md#f36 | Portable data dir |
| F4.1 | TODO | F4.md#f41 | Remove vault.key plaintext fallback |
| F4.2 | TODO | F4.md#f42 | Legacy layout detection |
| F4.3 | TODO | F4.md#f43 | Test vault helper and DB tests |
| F4.4 | TODO | F4.md#f44 | Zeroization |
| F4.5 | TODO | F4.md#f45 | Lockout counter in core |
| F4.6 | TODO | F4.md#f46 | Passphrase strength and PIN policy |
| F4.7 | TODO | F4.md#f47 | Atomic file writes |
| F5.1 | TODO | F5.md#f51 | Migration runner |
| F5.2 | TODO | F5.md#f52 | Sync table standard columns |
| F5.3 | TODO | F5.md#f53 | Unix millisecond timestamps |
| F5.4 | TODO | F5.md#f54 | Monotonic rev and change_log |
| F5.5 | TODO | F5.md#f55 | Central sync helper |
| F5.6 | TODO | F5.md#f56 | Command logs audit table |
| F5.7 | TODO | F5.md#f57 | Database convention guard tests |
| F5.8 | TODO | F5.md#f58 | Quality bars on data layer |
| F6.1 | DONE | F6.md#f61 | DEK 32-byte and dual key wrapping |
| F6.2 | DONE | F6.md#f62 | BIP-39 24-word recovery code |
| F6.3 | DONE | F6.md#f63 | Master password change via DEK re-wrapping |
| F6.4 | DONE | F6.md#f64 | Recovery code regeneration |
| F7.1 | DONE | F7.md | Clean unused/phantom commands |
| F7.2 | DONE | F7.md | Parameter & return type alignment |
| F7.3 | DONE | F7.md | Policy classes and command metadata |
| F7.4 | DONE | F7.md | Generated TypeScript bindings |
| F7.5 | DONE | F7.md | Frontend API wrappers |
| F7.6 | DONE | F7.md | IPC contract tests |
| F8.1 | DONE | F8.md | SessionState in core |
| F8.2 | DONE | F8.md | Remove caller_profile_id & is_owner from signatures |
| F8.3 | DONE | F8.md | Capability & role checks in core |
| F8.4 | DONE | F8.md | Super admin audit log for private access |
| F8.5 | DONE | F8.md | Session expiry & activity tracking |
| F9.1 | DONE | F9.md | Visibility filter in SQL queries |
| F9.2 | DONE | F9.md | Notes visibility isolation |
| F9.3 | DONE | F9.md | Row-level ownership checks |
| F10.1 | DONE | F10.md | 2-level lock screen UI |
| F10.2 | DONE | F10.md | Profile picker with public metadata |
| F10.3 | DONE | F10.md | PIN pad input component |
| F10.4 | DONE | F10.md | Recovery code unlock flow |
| F10.5 | DONE | F10.md | Auto-lock timers |
| F11.1 | DONE | F11.md | Backup format v2 |
| F11.2 | DONE | F11.md | Restore transaction & conflict handling |
| F11.3 | DONE | F11.md | Round-trip backup/restore tests |
| F11.4 | DONE | F11.md | Scheduled backup engine |
| F12.1 | IN_PROGRESS | F12.md | Module toggle enforcement |
| F12.2 | TODO | F12.md#f122 | Pro licensing client |
| F12.3 | TODO | F12.md#f123 | Feedback and crash report scrubber |
| F12.4 | TODO | F12.md#f124 | Updater integration |
| F13.1 | TODO | F13.md#f131 | AI provider abstraction |
| F13.2 | TODO | F13.md#f132 | AI privacy levels & context scrubber |
| F13.3 | TODO | F13.md#f133 | AI UI chat rendering |
| F14.1 | TODO | F14.md#f141 | TitleBar with window controls |
| F14.2 | TODO | F14.md#f142 | Responsive navigation drawer |
| F14.3 | TODO | F14.md#f143 | Dynamic dark/light theme |
| F14.4 | TODO | F14.md#f144 | Dynamic accent color system |
| F14.5 | TODO | F14.md#f145 | PageContainer & state components |
| F14.6 | TODO | F14.md#f146 | Settings registry |
| F15.1 | TODO | F15.md#f151 | Complete i18n dictionaries |
| F15.2 | TODO | F15.md#f152 | Type-safe t() helper |
| F15.3 | TODO | F15.md#f153 | Locale formatters |
| F16.1 | TODO | F16.md#f161 | Notes CRUD completion |
| F16.2 | TODO | F16.md#f162 | Notes export/import JSON |
| F17.1 | TODO | F17.md#f171 | Open source documentation |
| F17.2 | TODO | F17.md#f172 | Security policy & license notices |
| F18.1 | TODO | F18.md#f181 | new-app generator rewrite |
| F18.2 | TODO | F18.md#f182 | new-app E2E testing |
| F19.1 | TODO | F19.md#f191 | Android build verification |
| F20.1 | TODO | F20.md#f201 | Release DoD re-audit |
| F20.2 | TODO | F20.md#f202 | Benchmarks & budgets |
| F20.3 | TODO | F20.md#f203 | Release readiness report |

## Owner decisions (copied from TASK_FIX.md §3 and TASK_FEATURES.md §1)
| ID | Decision | Date | Changed? |
|---|---|---|---|
| D-1 | Schema `app.toml` contract = PRD v1.1 FR-1 in full + optional `[window]`. Validate/codegen all sections. | 2026-10-04 | No |
| D-2 | Only super_role may read other profiles' private rows (audited as PRIVATE_READ, badge shown). Non-super roles receive only private_summary aggregates. | 2026-10-04 | No |
| D-3 | Desktop launcher generated by xtask to `packaging/linux/<slug>.desktop`. Snap/Flatpak/Docker out of scope. | 2026-10-04 | No |
| D-4 | Pro scope v0.1.0 = Licensing client only (status, auth, trial, password reset, device revocation). ProTeamPanel & sync removed until v0.2.0. | 2026-10-04 | No |
| D-5 | No migration for pre-release vaults. Startup returns `VaultError::LegacyFormat`. | 2026-10-04 | No |
| D-6 | Publication hygiene: purge caterm binaries and move Notes/ to private repo before public push (owner executes). | 2026-10-04 | No |
| D-7 | Security defaults: Profile lock 5m, vault lock 15m, Master pwd >= 12 chars, PIN 4-8 digits, backoff after 5 attempts, 10 consecutive PIN fails lock vault. | 2026-10-04 | No |
| D-8 | Autonomous single continuous session with ledger persistence. | 2026-10-04 | No |
| D-9 | RBAC model: resource:action permissions, wildcard support, default roles super_admin, admin, member, viewer. | 2026-10-04 | No |
| D-10 | Backup is mandatory; `modules.backup = false` is invalid. | 2026-10-04 | No |
| D-11 | Brand is mandatory (`[brand]` required). | 2026-10-04 | No |
| D-12 | Pro package always generated in v0.2.0; Free/Pro feature gating via `[pro.features]`. | 2026-10-04 | No |
| D-13 | Measurable quality bars: stable, maintainable, scalable enforced at every gate. | 2026-10-04 | No |

## OWNER-VERIFY queue
| ID | Stage/task | What | Blocking? | Status | Owner output |
|---|---|---|---|---|---|
| OV-F0-1 | F0.3 | Section D (check real ~/.local/share/caframework) & GUI visual smoke test | No | PENDING | - |

## Questions to owner
| # | Question (Bahasa Indonesia) | Recommendation | Blocking? | Answer | Date |
|---|---|---|---|---|---|

## Handoff notes
### F0
F0 scaffolding and baseline audit underway on main branch. Toolchain verified (rustc 1.98.1, cargo 1.98.1, node 26.8.1, npm 11.19.0, cargo-deny 0.20.2, gitleaks, actionlint). Notes scaffolding and prd-amendments committed.
