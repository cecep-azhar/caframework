# Backend Contract & API Reference

This document describes the IPC communication contract between the Tauri backend (`caf-app` / `caf-core`) and the Svelte frontend (`caf-frontend`).

## Command Categories & Policies

Commands are grouped into policy classes:
1. **Public Commands**: Can be invoked before unlocking the vault or choosing a profile (e.g. `list_profiles_public`, `get_app_info`).
2. **Session / Profile Commands**: Require an active unlocked session (e.g. `list_notes`, `save_note`, `change_password`).
3. **Admin / Super Admin Commands**: Require elevated RBAC permissions (`super_admin` role) for cross-profile access.

## Error Model

All commands return standard structured errors matching the `CatermError` taxonomy defined in `docs/error-codes.md`.
