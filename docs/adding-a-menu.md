# Adding a New Feature / Menu in CAFramework

Panduan menambahkan menu / vertical slice baru dari Database hingga Frontend UI.

---

## Langkah 1: Tambahkan Definisi Menu di `app.toml`
```toml
[[menus]]
key = "tasks"
label = "Tasks"
href = "/tasks"
icon = "checklist"
description = "Manage daily tasks"
```
Jalankan codegen:
```bash
cargo run --package caf-xtask -- codegen --config app.toml
```

---

## Langkah 2: Buat Modul Backend di `caf-core`
1. Buat file `crates/caf-core/src/tasks.rs` dengan skema struct data:
   - Wajib sertakan `id`, `rev`, `created_at`, `updated_at`, `owner_profile_id`.
2. Daftarkan query SQL (SELECT, INSERT, UPDATE, DELETE).
3. Export modul di `crates/caf-core/src/lib.rs`.

---

## Langkah 3: Daftarkan Tauri IPC Command di `caf-app`
1. Tambahkan fungsi IPC di `crates/caf-app/src/commands.rs`:
   ```rust
   #[tauri::command]
   pub fn list_tasks(profile_id: String) -> Result<Vec<TaskRecord>, String> {
       caf_core::tasks::list_tasks(&profile_id).map_err(|e| e.to_string())
   }
   ```
2. Daftarkan command di `crates/caf-app/src/lib.rs` (`invoke_handler!`).
3. Daftarkan permission di `crates/caf-app/build.rs` dan `crates/caf-app/capabilities/default.json`.

---

## Langkah 4: Buat Halaman Frontend di `frontend/src/routes/`
1. Buat file `frontend/src/routes/tasks/+page.svelte`.
2. Gunakan Svelte 5 Runes (`$state`, `$derived`, `$effect`) dan panggil API via `@tauri-apps/api/core`.
3. Manfaatkan reusable component: `<EmptyState />`, `<LoadingState />`, `<ErrorState />`.
