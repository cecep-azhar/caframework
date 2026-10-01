# CAFramework v2 (Tauri + Rust + SvelteKit) Task Tracker

## 1. Project Setup
- [x] Initialize Tauri project with Rust backend and SvelteKit frontend (if not already done).
- [x] Configure `tauri.conf.json` for proper window constraints and permissions.
- [x] Set up TailwindCSS or equivalent for styling in SvelteKit.

## 2. Layout & Global Navigation
- [x] Implement Sidebar/Top bar for Global navigation (Dashboard, Hosts, Groups, Snippets, Settings).
- [x] Implement Sessions Tabs system (e.g., Ctrl+1..3 to switch active sessions).

## 3. Dashboard Welcome Screen
- [x] Design Welcome Page featuring E2EE, SFTP, Audit Log, AI Terminal, and Dedikasi.
- [x] Add "Report Issue" button opening a modal with a mailto link or form to `cecep.azhtech@gmail.com`.

## 4. Hosts Management
- [x] Create UI for listing Hosts.
- [x] Implement "Add Host" modal (Fields: Label, IP, Port, Username, Password/Key, Tags).
- [x] Implement Rust backend commands to store/retrieve Host configurations (Local-First SQLite/JSON).

## 5. Terminal Split Pane (Core UX)
- [x] Integrate xterm.js within a Svelte component.
- [x] Implement split grid system (up to 3-4 terminal instances active).
- [x] Display connection details (Path/IP) on active terminal header.
- [x] Bind Rust SSH client (e.g., `russh` or `ssh2`) to xterm.js frontend.

## 6. Groups Management
- [x] Create UI for Groups.
- [x] Implement "Create Group" modal (Fields: Name, Color Swatch, Multiselect Hosts).
- [x] Link backend logic to map Hosts to Groups.

## 7. Snippets System
- [x] Create UI for Snippets.
- [x] Implement 2-step creation modal:
      - Step 1: Bash Command entry.
      - Step 2: Label, Description, Tags entry.
- [x] Add ability to execute snippet on active terminal session.

## 8. Settings
- [x] Implement Settings view.
- [x] Add 'Update' section.
- [x] Add 'Cloud Sync E2EE' section (Placeholder $1/mo and team $1 marked as "Next Feature/Disabled", not a standard Pro upgrade).

## 9. Security & DB (Zero-Knowledge)
- [x] Ensure SQLite database uses strong encryption (e.g., `sqlcipher` or similar Rust crates).
- [x] Implement minimum 8-character password requirement for vault unlock.
## 10. Audit & Completion
- [x] Audited and completed Hosts, Groups, Snippets, and SSH Keys persistence to SQLite. Updated Hosts toolbar to match screenshot requirements.
## 11. Teams
- [x] Create Team model in `caf-core` with SQLite storage (fields: id, name, color, avatar, members, host_ids, group_ids).
- [x] Implement Tauri commands for list, save, delete.
- [x] Create `frontend/src/routes/teams/+page.svelte` matching Hosts/Groups UI.

- [x] **Agent 4 - Fitur Baru: Command Logs**: 
  - Backend: Log events to encrypted SQLite (caf-core/src/audit.rs).
  - Track: PTY hook (commands), tunnels, vault, ssh keys.
  - Privacy: Mask passwords and secrets automatically.
  - Route: frontend/src/routes/command-logs/+page.svelte (scrollable, filter, search, export).

## 12. Investigations
- [x] Create Investigations model in `caf-core` with SQLite storage (fields: id, title, host_id, status, notes, evidence).
- [x] Implement Tauri commands `list_investigations`, `save_investigation`, `delete_investigation`.
- [x] Create `frontend/src/routes/investigations/+page.svelte` featuring a list of incidents, timeline detail view, and "New Investigation" tied to hosts/time range.

## 12. CLI (caframeworkctl)
- [x] Implement CLI parity (host, vault, connect, tunnel, monitor, key, audit) in caf-cli/src/main.rs calling caf-core.

## 13. Prompt Studio (AI Ops Assistant & Automation)
- [x] Backend AI module di `caf-core/src/ai.rs` (OpenAI/Anthropic/Ollama/9router API client dengan built-in fallback heuristic plan generator).
- [x] Skema SQLite: tabel `ai_settings` untuk konfigurasi provider, base URL, API key, dan model name.
- [x] Tauri commands: `get_ai_settings`, `save_ai_settings`, `ai_generate_plan`, `ai_execute_step` terdaftar di runtime, build.rs, dan capabilities default.json.
- [x] Prompt Studio UI di `frontend/src/routes/prompt-studio/+page.svelte` dengan preset Ubuntu Laravel dev, Docker, Node.js, UFW.
- [x] Human-in-the-loop plan confirmation: review langkah, checklist per langkah, visualisasi sudo warning sebelum eksekusi.
- [x] Eksekusi SSH non-interactive berurutan dengan output live streaming dan pelacakan audit di `command_logs` (event `AI_AUTOMATION`).

## 14. WinSCP Feature Parity Build-out
- [ ] Fitur #1: Dukungan Protokol Non-SSH (SCP, FTP/FTPS, WebDAV, S3, RemoteFileSystem trait)
- [ ] Fitur #2: Sinkronisasi Direktori (Two-way/one-way sync + file watcher notify)
- [ ] Fitur #3: Manajemen Antrean Transfer (Queue & Resume + transfers.rs)
- [ ] Fitur #4: Remote File Editor & Live Edit (CodeMirror 6 + RemoteFileEditor.svelte)
- [ ] Fitur #5: Verifikasi Integritas (Checksum & Hash remote + local + UI)
- [ ] Fitur #6: Pencarian File Remote (find + recursive walk SFTP + UI panel)
- [ ] Fitur #7: Kompresi & Ekstraksi Remote (tar/zip via exec session + UI)


