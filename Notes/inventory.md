# Inventory Baseline CAFramework v2.1.12 (`601693c091a191c1cbc3e959625ea33e15faee0a`)

## 1. Rust Core Modules (`crates/caf-core/src/`)
Total: 30 modules

| Module | Classification | Notes |
| --- | --- | --- |
| `ai.rs` | GENERALISE | Keep BYO OpenAI/Ollama/Hosted AI assistant & guardrails |
| `audit.rs` | KEEP | Audit logs |
| `backup.rs` | GENERALISE | Encrypted backup/restore format v1 |
| `crash.rs` | GENERALISE | Panic hook, local dump 0600, scrubber |
| `db.rs` | GENERALISE | SQLCipher, migrations, change_log sync-ready |
| `error.rs` | GENERALISE | Generic error codes with prefix from config (`CAF-*`) |
| `feedback.rs` | GENERALISE | GCC proxy feedback submission |
| `paths.rs` | GENERALISE | App data directories & storage paths |
| `prefs.rs` | GENERALISE | UI & generic app preferences |
| `pro.rs` | GENERALISE | License activation, device seat, session management |
| `secret.rs` | KEEP | Zeroize, secrecy helpers |
| `vault.rs` | GENERALISE | Master password, Argon2id, DEK encryption |
| `ftp.rs` | REMOVE | Specific to FTP |
| `groups.rs` | REMOVE | Specific to host grouping |
| `investigations.rs` | REMOVE | Terminal investigation logs |
| `keys.rs` | REMOVE | SSH Key management |
| `local_fs.rs` | REMOVE | Local FS explorer for SFTP |
| `monitor.rs` | REMOVE | Server monitoring daemon/metrics |
| `s3.rs` | REMOVE | S3 storage explorer |
| `scp.rs` | REMOVE | SCP client |
| `sftp.rs` | REMOVE | SFTP client |
| `snippets.rs` | REMOVE | Terminal snippets |
| `ssh.rs` | REMOVE | SSH client, PTY, libssh2 |
| `store.rs` | REMOVE | Host connection store |
| `sync.rs` | REMOVE | Replaced with generic sync-ready conventions & change_log |
| `teams.rs` | REMOVE | Fold generic team primitives into Pro if needed |
| `tunnels.rs` | REMOVE | SSH Port forwarding/tunnels |
| `vfs.rs` | REMOVE | Virtual filesystem for remote storage |
| `webdav.rs` | REMOVE | WebDAV client |
| `lib.rs` | GENERALISE | Root exports |

## 2. Frontend Routes (`frontend/src/routes/`)
Total: 15 routes

| Route | Classification | Notes |
| --- | --- | --- |
| `+layout.svelte` / `+layout.ts` | GENERALISE | Shell, title bar, sidebar, lock, palette, toasts |
| `+page.svelte` | GENERALISE | Home dashboard / Sample route |
| `settings` | GENERALISE | Dynamic modular settings registry |
| `prompt-studio` | GENERALISE | Folded into AI Assistant module |
| `contribution` | GENERALISE | Optional Support / Sponsor page |
| `teams` | GENERALISE | Folded into Pro if enabled |
| `command-logs` | REMOVE | Terminal command logs |
| `groups` | REMOVE | Host grouping |
| `investigations` | REMOVE | Incident investigation logs |
| `monitoring` | REMOVE | Server monitoring |
| `port-forwarding` | REMOVE | SSH tunnels |
| `session` | REMOVE | SSH Terminal session |
| `sftp` | REMOVE | Remote SFTP browser |
| `snippets` | REMOVE | Terminal snippets |
| `ssh-keys` | REMOVE | SSH Key generator & manager |

## 3. Frontend Components (`frontend/src/lib/components/`)
Total: 31 components

| Component | Classification |
| --- | --- |
| `PageContainer.svelte`, `PageHeader.svelte`, `CommandPalette.svelte`, `NotificationCenter.svelte`, `LockScreen.svelte`, `UpdateToast.svelte`, `AboutModal.svelte`, `FeedbackModal.svelte`, `FeedbackWidget.svelte`, `CrashReportModal.svelte`, `LanguageSwitcher.svelte`, `ProfileMenu.svelte`, `ProfileAvatar.svelte`, `AvatarPicker.svelte`, `Logo.svelte`, `AiChatPanel.svelte`, `AiSettingsForm.svelte`, `ProLoginForm.svelte`, `ProTeamPanel.svelte`, `SponsorWall.svelte`, `GridFlowBackground.svelte` | KEEP / GENERALISE |
| `SessionViewport.svelte`, `TerminalPane.svelte`, `TerminalAutocomplete.svelte`, `SessionFileManager.svelte`, `RemoteFileEditor.svelte`, `HostDetailPanel.svelte`, `OsIcon.svelte`, `DirectorySync.svelte`, `VpsRecommendation.svelte`, `WorkspaceMenu.svelte` | REMOVE |

## 4. Frontend Stores (`frontend/src/lib/stores/`)
Total: 16 stores

| Store | Classification |
| --- | --- |
| `theme.svelte.ts`, `appearance.svelte.ts`, `uiNotifications.svelte.ts`, `commandPalette.svelte.ts`, `profile.svelte.ts`, `pro.svelte.ts`, `aiChat.svelte.ts`, `feedbackStore.svelte.ts`, `updater.svelte.ts` | KEEP / GENERALISE |
| `activeSession.svelte.ts`, `sessionTabs.svelte.ts`, `sessionView.svelte.ts`, `monitorStore.svelte.ts`, `terminalPrefs.svelte.ts`, `hostPrefs.svelte.ts`, `workspaceStore.svelte.ts` | REMOVE |

## 5. Commands Baseline
Total `#[tauri::command]`: 112 commands in CAFramework v2.1.12 baseline.
Target CAFramework after S2: ~25 core generic commands (vault, profiles, prefs, backup, crash, ai, pro, updater, feedback, sample notes).
