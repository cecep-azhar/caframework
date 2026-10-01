//! Tauri binding layer. Deliberately empty of business logic (K2-1): every
//! `#[tauri::command]` added here must do nothing but deserialize -> call
//! one `caf_core` function -> serialize. No demo commands (anti-regresi T-19).

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented
)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::todo,
        clippy::unimplemented
    )
)]

mod commands;
mod window;

use serde::Serialize;
use tauri::Emitter;

/// Payload for the `ssh://output` event. One event per chunk the PTY reader thread produces.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshOutputEvent {
    session_id: String,
    data: String,
}

/// Payload for `ssh://closed`, emitted once when a PTY channel reaches EOF.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshClosedEvent {
    session_id: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HostOsDetectedEvent {
    host_id: String,
    os: String,
}

/// Bridges `caf_core`'s GUI-free event sink to Tauri's event bus. Registering this is what
/// switches PTY delivery from "frontend polls `ssh_read` 25x/second" to push — the core buffers
/// nothing once a sink exists, so bytes are delivered exactly once.
fn install_ssh_event_bridge(app: &tauri::AppHandle) {
    let handle = app.clone();
    caf_core::ssh::set_event_sink(move |event| match event {
        caf_core::ssh::SshEvent::Output { session_id, data } => {
            let _ = handle.emit("ssh://output", SshOutputEvent { session_id, data });
        }
        caf_core::ssh::SshEvent::Closed { session_id } => {
            let _ = handle.emit("ssh://closed", SshClosedEvent { session_id });
        }
        caf_core::ssh::SshEvent::OsDetected { host_id, os } => {
            let _ = handle.emit("host:os_detected", HostOsDetectedEvent { host_id, os });
        }
    });
}

/// `start` should be captured as close to `main()`'s first line as possible by the
/// caller, so the printed duration approximates true cold start (REQ-02). Prints
/// `CATERM_COLD_START_MS=<n>` once the backend considers itself ready (main window
/// built). This is a *backend-ready* proxy for now — Fase 0 has no frontend IPC yet
/// (K2-1) and no unlock screen (Fase 3), so it is not yet REQ-02's literal
/// "cold start -> jendela unlock". `T2-BOOT-06` evidence documents this scope honestly;
/// the harness will be repointed at a real frontend-ready signal once Fase 3 ships one.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    run_with_start(std::time::Instant::now());
}

pub fn run_with_start(start: std::time::Instant) {
    // As early as possible, before any other thread can panic: opt-in, local-first crash
    // reporting (caframework-crash-reporting-spec-v1.md). Zero telemetry by default — this only
    // ever writes a scrubbed dump to disk; nothing is sent until the user reviews and approves
    // it on the next launch (see commands::get_pending_crash_report).
    caf_core::crash::install_panic_hook();

    // Unrecoverable: if the Tauri runtime itself fails to start, the process has no
    // useful state to continue in. This is the one sanctioned exception to the
    // zero-panic policy (REQ-04) outside #[cfg(test)] — see ledger-v2.md KA-05.
    // `disallowed_methods` is also allowed here: `tauri::generate_context!()` expands to
    // code that calls `std::process::exit` internally (Tauri's own codegen, not ours) —
    // the lint attributes that call to this statement's span.
    #[allow(clippy::expect_used, clippy::disallowed_methods)]
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        // The frontend imports @tauri-apps/plugin-dialog (native save/open dialogs on the
        // Command Logs CSV export and the Snippets import/export) and @tauri-apps/plugin-fs
        // (writeTextFile/readTextFile against the path the user picked in that dialog). Neither
        // plugin was registered here (C-17) even though caf-app already depended on
        // tauri-plugin-dialog, and tauri-plugin-fs wasn't even a dependency — both features
        // silently failed with a "plugin not found" error at the call site.
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            use tauri::Manager;
            let _ = app.get_webview_window("main").map(|w| w.set_focus());
        }))
        .setup(move |app| {
            window::create_main_window(app)?;
            install_ssh_event_bridge(app.handle());

            // Desktop only: the updater has no mobile implementation (see Dockerfile.android).
            #[cfg(desktop)]
            app.handle()
                .plugin(tauri_plugin_updater::Builder::new().build())?;

            #[cfg(target_os = "linux")]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    window
                        .with_webview(|webview| {
                            use webkit2gtk::{SettingsExt, WebViewExt};
                            let inner = webview.inner();
                            if let Some(settings) = inner.settings() {
                                let enable_gpu = caf_core::prefs::load_performance_prefs()
                                    .map(|p| p.gpu_acceleration)
                                    .unwrap_or(false);
                                settings.set_enable_webgl(enable_gpu);
                            }
                        })
                        .ok();
                }
            }

            println!("CATERM_COLD_START_MS={}", start.elapsed().as_millis());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_hosts,
            commands::save_host,
            commands::delete_host,
            commands::list_groups,
            commands::save_group,
            commands::delete_group,
            commands::list_teams,
            commands::save_team,
            commands::delete_team,
            commands::list_investigations,
            commands::save_investigation,
            commands::delete_investigation,
            commands::list_snippets,
            commands::save_snippet,
            commands::delete_snippet,
            commands::list_tunnels,
            commands::save_tunnel,
            commands::delete_tunnel,
            commands::start_tunnel,
            commands::stop_tunnel,
            commands::poll_active_metrics,
            commands::list_remote_dir,
            commands::search_remote_files,
            commands::read_remote_file,
            commands::write_remote_file,
            commands::mkdir_remote_dir,
            commands::delete_remote_file,
            commands::sftp_stat,
            commands::sftp_chmod,
            commands::sftp_upload,
            commands::sftp_download,
            commands::sftp_cancel,
            commands::sftp_rename,
            commands::sftp_copy,
            commands::sftp_compress,
            commands::sftp_extract,
            commands::calculate_remote_checksum,
            commands::calculate_local_checksum,
            commands::compare_file_checksums,
            commands::local_list_dir,
            commands::local_stat,
            commands::local_mkdir,
            commands::local_delete,
            commands::local_rename,
            commands::local_read_file,
            commands::local_write_file,
            commands::export_encrypted_backup,
            commands::import_encrypted_backup,
            commands::list_keys,
            commands::generate_key,
            commands::import_key,
            commands::delete_key,
            commands::deploy_public_key,
            commands::validate_vault_password,
            commands::is_vault_initialized,
            commands::reset_vault,
            commands::lock_vault,
            commands::change_master_password,
            commands::get_command_logs,
            commands::ssh_connect,
            commands::ssh_write,
            commands::ssh_read,
            commands::ssh_resize,
            commands::ssh_disconnect,
            commands::detect_host_os,
            commands::get_ai_settings,
            commands::save_ai_settings,
            commands::ai_generate_plan,
            commands::ai_chat,
            commands::ai_execute_step,
            commands::plan_sync,
            commands::execute_sync,
            commands::start_watch,
            commands::stop_watch,
            commands::list_watches,
            commands::window_minimize,
            commands::window_maximize,
            commands::window_close,
            commands::window_start_dragging,
            commands::open_external_url,
            commands::get_performance_prefs,
            commands::set_performance_prefs,
            commands::pro_status,
            commands::pro_server_available,
            commands::pro_register,
            commands::pro_resend_verification,
            commands::pro_forgot_password,
            commands::pro_login,
            commands::pro_commit_pending,
            commands::pro_sync,
            commands::pro_start_trial,
            commands::pro_account,
            commands::pro_revoke_device,
            commands::pro_logout,
            commands::pro_team,
            commands::pro_team_invite,
            commands::pro_team_cancel_invite,
            commands::pro_team_remove_member,
            commands::pro_team_accept,
            commands::pro_team_decline,
            commands::pro_team_leave,
            commands::pro_ai_usage,
            commands::get_pending_crash_report,
            commands::submit_crash_report,
            commands::dismiss_crash_report,
            commands::submit_feedback,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
