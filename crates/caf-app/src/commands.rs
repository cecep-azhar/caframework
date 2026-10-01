//! Thin Tauri command bindings (K2-1). Each command below only
//! deserializes its arguments, calls exactly one `caf_core` function, and
//! returns its result — no business logic here. See `tests/arch.rs` in
//! `caf-core` for the guard that enforces this.

use caf_core::{
    CatermError, ai, audit, backup, feedback, groups, investigations, keys, monitor, prefs, sftp,
    snippets, ssh, store, sync, teams, tunnels, vault, vfs,
};

async fn run_blocking<F, R>(f: F) -> Result<R, CatermError>
where
    F: FnOnce() -> Result<R, CatermError> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| caf_core::error::IoError::Generic(format!("Task join error: {e}")))?
}

#[tauri::command]
pub async fn window_minimize(#[allow(unused_variables)] window: tauri::Window) {
    #[cfg(not(target_os = "android"))]
    let _ = window.minimize();
}

#[tauri::command]
pub async fn window_maximize(#[allow(unused_variables)] window: tauri::Window) {
    #[cfg(not(target_os = "android"))]
    {
        if let Ok(max) = window.is_maximized() {
            if max {
                let _ = window.unmaximize();
            } else {
                let _ = window.maximize();
            }
        } else {
            let _ = window.maximize();
        }
    }
}

#[tauri::command]
pub async fn window_close(#[allow(unused_variables)] window: tauri::Window) {
    #[cfg(not(target_os = "android"))]
    let _ = window.close();
}

#[tauri::command]
pub async fn window_start_dragging(#[allow(unused_variables)] window: tauri::Window) {
    #[cfg(not(target_os = "android"))]
    let _ = window.start_dragging();
}

#[tauri::command]
pub async fn list_tunnels() -> Result<Vec<tunnels::TunnelRecord>, CatermError> {
    run_blocking(tunnels::list_tunnels).await
}

#[tauri::command]
pub async fn save_tunnel(
    input: tunnels::TunnelInput,
) -> Result<tunnels::TunnelRecord, CatermError> {
    run_blocking(move || tunnels::save_tunnel(input)).await
}

#[tauri::command]
pub async fn delete_tunnel(id: String) -> Result<(), CatermError> {
    run_blocking(move || tunnels::delete_tunnel(&id)).await
}

#[tauri::command]
pub async fn start_tunnel(id: String) -> Result<(), CatermError> {
    run_blocking(move || tunnels::start_tunnel(&id)).await
}

#[tauri::command]
pub async fn stop_tunnel(id: String) -> Result<(), CatermError> {
    run_blocking(move || tunnels::stop_tunnel(&id)).await
}
#[tauri::command]
pub async fn list_remote_dir(
    host_id: String,
    remote_path: String,
) -> Result<Vec<sftp::SftpFileEntry>, CatermError> {
    run_blocking(move || vfs::get_remote_fs(&host_id)?.list_dir(&remote_path)).await
}

#[tauri::command]
pub async fn search_remote_files(
    host_id: String,
    base_path: String,
    pattern: String,
    max_results: Option<usize>,
    min_size: Option<u64>,
    max_size: Option<u64>,
) -> Result<Vec<sftp::RemoteSearchItem>, CatermError> {
    run_blocking(move || {
        sftp::search_remote_files(
            &host_id,
            &base_path,
            &pattern,
            max_results.unwrap_or(100),
            min_size,
            max_size,
        )
    })
    .await
}

#[tauri::command]
pub async fn read_remote_file(
    host_id: String,
    remote_path: String,
) -> Result<Vec<u8>, CatermError> {
    run_blocking(move || vfs::get_remote_fs(&host_id)?.read_file(&remote_path)).await
}

#[tauri::command]
pub async fn write_remote_file(
    host_id: String,
    remote_path: String,
    data: Vec<u8>,
) -> Result<(), CatermError> {
    run_blocking(move || vfs::get_remote_fs(&host_id)?.write_file(&remote_path, &data)).await
}

#[tauri::command]
pub async fn mkdir_remote_dir(host_id: String, remote_path: String) -> Result<(), CatermError> {
    run_blocking(move || vfs::get_remote_fs(&host_id)?.mkdir(&remote_path)).await
}

#[tauri::command]
pub async fn delete_remote_file(
    host_id: String,
    remote_path: String,
    is_dir: Option<bool>,
    recursive: Option<bool>,
) -> Result<(), CatermError> {
    run_blocking(move || {
        vfs::get_remote_fs(&host_id)?.delete(
            &remote_path,
            is_dir.unwrap_or(false),
            recursive.unwrap_or(false),
        )
    })
    .await
}

#[tauri::command]
pub async fn sftp_stat(
    host_id: String,
    remote_path: String,
) -> Result<sftp::SftpFileEntry, CatermError> {
    run_blocking(move || vfs::get_remote_fs(&host_id)?.stat(&remote_path)).await
}

#[tauri::command]
pub async fn sftp_chmod(
    host_id: String,
    remote_path: String,
    mode: u32,
) -> Result<(), CatermError> {
    run_blocking(move || vfs::get_remote_fs(&host_id)?.chmod(&remote_path, mode)).await
}

fn emit_progress_fn(
    app: &tauri::AppHandle,
) -> std::sync::Arc<parking_lot::Mutex<impl FnMut(sftp::SftpProgressPayload) + Send + 'static>> {
    use tauri::Emitter;
    let app_clone = app.clone();
    std::sync::Arc::new(parking_lot::Mutex::new(
        move |p: sftp::SftpProgressPayload| {
            let _ = app_clone.emit("sftp-progress", p);
        },
    ))
}

#[tauri::command]
pub async fn sftp_upload(
    app: tauri::AppHandle,
    host_id: String,
    local_path: String,
    remote_path: String,
    transfer_id: String,
) -> Result<(), CatermError> {
    let cb = emit_progress_fn(&app);
    run_blocking(move || {
        vfs::get_remote_fs(&host_id)?.upload(&local_path, &remote_path, &transfer_id, cb)
    })
    .await
}

#[tauri::command]
pub async fn sftp_download(
    app: tauri::AppHandle,
    host_id: String,
    remote_path: String,
    local_path: String,
    transfer_id: String,
) -> Result<(), CatermError> {
    let cb = emit_progress_fn(&app);
    run_blocking(move || {
        vfs::get_remote_fs(&host_id)?.download(&remote_path, &local_path, &transfer_id, cb)
    })
    .await
}

#[tauri::command]
pub async fn sftp_cancel(transfer_id: String) -> Result<(), CatermError> {
    sftp::cancel_transfer(&transfer_id)
}

#[tauri::command]
pub async fn sftp_compress(
    host_id: String,
    parent_dir: String,
    items: Vec<String>,
    archive_name: String,
) -> Result<(), CatermError> {
    run_blocking(move || sftp::compress_remote(&host_id, &parent_dir, items, &archive_name)).await
}

#[tauri::command]
pub async fn sftp_extract(
    host_id: String,
    archive_path: String,
    dest_dir: String,
) -> Result<(), CatermError> {
    run_blocking(move || sftp::extract_remote(&host_id, &archive_path, &dest_dir)).await
}

#[tauri::command]
pub async fn local_list_dir(
    path: String,
) -> Result<Vec<caf_core::local_fs::LocalFileEntry>, CatermError> {
    run_blocking(move || caf_core::local_fs::local_list_dir(&path)).await
}

#[tauri::command]
pub async fn local_stat(
    path: String,
) -> Result<caf_core::local_fs::LocalFileEntry, CatermError> {
    run_blocking(move || caf_core::local_fs::local_stat(&path)).await
}

#[tauri::command]
pub async fn local_mkdir(path: String) -> Result<(), CatermError> {
    run_blocking(move || caf_core::local_fs::local_mkdir(&path)).await
}

#[tauri::command]
pub async fn local_delete(
    path: String,
    is_dir: Option<bool>,
    recursive: Option<bool>,
) -> Result<(), CatermError> {
    run_blocking(move || {
        caf_core::local_fs::local_delete(
            &path,
            is_dir.unwrap_or(false),
            recursive.unwrap_or(false),
        )
    })
    .await
}

#[tauri::command]
pub async fn local_rename(old_path: String, new_path: String) -> Result<(), CatermError> {
    run_blocking(move || caf_core::local_fs::local_rename(&old_path, &new_path)).await
}

#[tauri::command]
pub async fn local_read_file(path: String) -> Result<Vec<u8>, CatermError> {
    run_blocking(move || caf_core::local_fs::local_read_file(&path)).await
}

#[tauri::command]
pub async fn local_write_file(path: String, data: Vec<u8>) -> Result<(), CatermError> {
    run_blocking(move || caf_core::local_fs::local_write_file(&path, &data)).await
}

#[tauri::command]
pub async fn sftp_rename(
    host_id: String,
    old_path: String,
    new_path: String,
) -> Result<(), CatermError> {
    run_blocking(move || vfs::get_remote_fs(&host_id)?.rename(&old_path, &new_path)).await
}

#[tauri::command]
pub async fn sftp_copy(
    host_id: String,
    src_path: String,
    dst_path: String,
) -> Result<(), CatermError> {
    run_blocking(move || sftp::copy_remote_file(&host_id, &src_path, &dst_path)).await
}

#[tauri::command]
pub async fn calculate_remote_checksum(
    host_id: String,
    path: String,
    algorithm: String,
) -> Result<String, CatermError> {
    run_blocking(move || sftp::calculate_remote_checksum(&host_id, &path, &algorithm)).await
}

#[tauri::command]
pub async fn calculate_local_checksum(
    path: String,
    algorithm: String,
) -> Result<String, CatermError> {
    run_blocking(move || {
        caf_core::local_fs::calculate_local_checksum(std::path::Path::new(&path), &algorithm)
    })
    .await
}

#[tauri::command]
pub async fn compare_file_checksums(
    host_id: String,
    remote_path: String,
    local_path: String,
    algorithm: String,
) -> Result<sftp::ChecksumComparison, CatermError> {
    run_blocking(move || {
        sftp::compare_file_checksums(&host_id, &remote_path, &local_path, &algorithm)
    })
    .await
}
#[tauri::command]
pub async fn export_encrypted_backup(passphrase: String) -> Result<String, CatermError> {
    run_blocking(move || backup::export_encrypted_backup(&passphrase)).await
}

#[tauri::command]
pub async fn import_encrypted_backup(
    encrypted_b64: String,
    passphrase: String,
) -> Result<usize, CatermError> {
    run_blocking(move || backup::import_encrypted_backup(&encrypted_b64, &passphrase)).await
}
#[tauri::command]
pub async fn list_keys() -> Result<Vec<keys::KeyRecord>, CatermError> {
    run_blocking(keys::list_keys).await
}

#[tauri::command]
pub async fn poll_active_metrics() -> Result<Vec<monitor::HostMetrics>, CatermError> {
    run_blocking(monitor::poll_active_metrics).await
}

#[tauri::command]
pub async fn generate_key(input: keys::KeyInput) -> Result<keys::KeyRecord, CatermError> {
    run_blocking(move || keys::generate_key(input)).await
}

#[tauri::command]
pub async fn import_key(
    name: String,
    private_key_pem: String,
    passphrase: Option<String>,
) -> Result<keys::KeyRecord, CatermError> {
    run_blocking(move || keys::import_key(&name, &private_key_pem, passphrase.as_deref())).await
}

#[tauri::command]
pub async fn delete_key(id: String) -> Result<(), CatermError> {
    run_blocking(move || keys::delete_key(&id)).await
}

#[tauri::command]
pub async fn deploy_public_key(host_id: String, key_id: String) -> Result<(), CatermError> {
    run_blocking(move || keys::deploy_public_key(&host_id, &key_id)).await
}

#[tauri::command]
pub async fn list_hosts() -> Result<Vec<store::HostRecord>, CatermError> {
    run_blocking(store::list_hosts).await
}

#[tauri::command]
pub async fn save_host(input: store::HostInput) -> Result<store::HostRecord, CatermError> {
    run_blocking(move || store::save_host(input)).await
}

#[tauri::command]
pub async fn delete_host(id: String) -> Result<(), CatermError> {
    run_blocking(move || store::delete_host(&id)).await
}

#[tauri::command]
pub async fn list_groups() -> Result<Vec<groups::GroupRecord>, CatermError> {
    run_blocking(groups::list_groups).await
}

#[tauri::command]
pub async fn save_group(input: groups::GroupInput) -> Result<groups::GroupRecord, CatermError> {
    run_blocking(move || groups::save_group(input)).await
}

#[tauri::command]
pub async fn delete_group(id: String) -> Result<(), CatermError> {
    run_blocking(move || groups::delete_group(&id)).await
}

#[tauri::command]
pub async fn list_snippets() -> Result<Vec<snippets::SnippetRecord>, CatermError> {
    run_blocking(snippets::list_snippets).await
}

#[tauri::command]
pub async fn save_snippet(
    input: snippets::SnippetInput,
) -> Result<snippets::SnippetRecord, CatermError> {
    run_blocking(move || snippets::save_snippet(input)).await
}

#[tauri::command]
pub async fn delete_snippet(id: String) -> Result<(), CatermError> {
    run_blocking(move || snippets::delete_snippet(&id)).await
}

#[tauri::command]
pub async fn validate_vault_password(password: String) -> Result<(), CatermError> {
    run_blocking(move || vault::unlock_vault(&password)).await
}

#[tauri::command]
pub async fn get_command_logs(
    host_id: Option<String>,
    search: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<audit::CommandLog>, CatermError> {
    run_blocking(move || audit::get_logs(host_id.as_deref(), search.as_deref(), limit)).await
}

#[tauri::command]
pub async fn is_vault_initialized() -> Result<bool, CatermError> {
    run_blocking(vault::is_vault_initialized).await
}

#[tauri::command]
pub async fn reset_vault() -> Result<(), CatermError> {
    run_blocking(vault::reset_vault).await
}

#[tauri::command]
pub async fn lock_vault() -> Result<(), CatermError> {
    run_blocking(vault::lock_vault).await
}

#[tauri::command]
pub async fn change_master_password(
    current_password: String,
    new_password: String,
) -> Result<(), CatermError> {
    run_blocking(move || vault::change_master_password(&current_password, &new_password)).await
}

#[tauri::command]
pub async fn ssh_connect(host_id: String) -> Result<ssh::SshSession, CatermError> {
    run_blocking(move || ssh::connect(&host_id)).await
}

#[tauri::command]
pub async fn ssh_write(session_id: String, data: String) -> Result<String, CatermError> {
    // Regression for C-08: this used to be a synchronous command, which Tauri v2 runs on the
    // main thread. ssh::write() calls audit::log_event() on Enter, which opens a database
    // connection — before the C-08 raw-key fix that alone cost ~156ms, stalling the whole UI
    // on every keystroke that submitted a line. Routing it through spawn_blocking (like every
    // other SSH/DB command here) keeps it off the main thread regardless.
    run_blocking(move || ssh::write(&session_id, &data)).await
}

#[tauri::command]
pub async fn ssh_read(session_id: String) -> Result<String, CatermError> {
    run_blocking(move || ssh::read(&session_id)).await
}

#[tauri::command]
pub async fn ssh_resize(session_id: String, cols: u16, rows: u16) -> Result<(), CatermError> {
    run_blocking(move || ssh::resize(&session_id, cols, rows)).await
}

#[tauri::command]
pub async fn ssh_disconnect(session_id: String) -> Result<(), CatermError> {
    run_blocking(move || ssh::disconnect(&session_id)).await
}

#[tauri::command]
pub async fn detect_host_os(host_id: String) -> Result<String, CatermError> {
    run_blocking(move || ssh::detect_host_os(&host_id)).await
}

#[tauri::command]
pub async fn list_teams() -> Result<Vec<teams::TeamRecord>, CatermError> {
    run_blocking(teams::list_teams).await
}

#[tauri::command]
pub async fn save_team(input: teams::TeamInput) -> Result<teams::TeamRecord, CatermError> {
    run_blocking(move || teams::save_team(input)).await
}

#[tauri::command]
pub async fn delete_team(id: String) -> Result<(), CatermError> {
    run_blocking(move || teams::delete_team(&id)).await
}

#[tauri::command]
pub async fn list_investigations() -> Result<Vec<investigations::InvestigationRecord>, CatermError>
{
    run_blocking(investigations::list_investigations).await
}

#[tauri::command]
pub async fn save_investigation(
    input: investigations::InvestigationInput,
) -> Result<investigations::InvestigationRecord, CatermError> {
    run_blocking(move || investigations::save_investigation(input)).await
}

#[tauri::command]
pub async fn delete_investigation(id: String) -> Result<(), CatermError> {
    run_blocking(move || investigations::delete_investigation(&id)).await
}

#[tauri::command]
pub async fn get_ai_settings() -> Result<ai::AiSettings, CatermError> {
    run_blocking(ai::get_ai_settings).await
}

#[tauri::command]
pub async fn save_ai_settings(
    settings: Option<ai::AiSettings>,
    input: Option<ai::AiSettings>,
) -> Result<ai::AiSettings, CatermError> {
    let s = settings.or(input).unwrap_or_default();
    run_blocking(move || ai::save_ai_settings(s)).await
}

#[tauri::command]
pub async fn ai_generate_plan(
    goal: String,
    host_id: Option<String>,
    hosted: Option<bool>,
) -> Result<ai::AiExecutionPlan, CatermError> {
    run_blocking(move || ai::generate_plan(&goal, host_id.as_deref(), hosted.unwrap_or(false)))
        .await
}

#[tauri::command]
pub async fn ai_chat(
    messages: Vec<ai::AiChatMessage>,
    host_label: Option<String>,
    hosted: Option<bool>,
) -> Result<ai::AiChatReply, CatermError> {
    run_blocking(move || ai::chat(messages, host_label.as_deref(), hosted.unwrap_or(false))).await
}

// ---- Hosted AI (CAFramework Pro pooled quota via OmniRoute) -------------------------------------

#[tauri::command]
pub async fn pro_ai_usage() -> Result<caf_core::pro::ProAiUsage, CatermError> {
    run_blocking(caf_core::pro::ai_usage).await
}

// ---- Crash reporting (opt-in, local-first — caframework-crash-reporting-spec-v1.md) -------------

#[tauri::command]
pub async fn get_pending_crash_report()
-> Result<Option<caf_core::crash::ScrubbedCrashReport>, CatermError> {
    run_blocking(caf_core::crash::pending_crash_report).await
}

#[tauri::command]
pub async fn submit_crash_report(report_id: String) -> Result<(), CatermError> {
    run_blocking(move || caf_core::crash::submit_crash_report(&report_id)).await
}

#[tauri::command]
pub async fn dismiss_crash_report(
    report_id: String,
    never_again: Option<bool>,
) -> Result<(), CatermError> {
    run_blocking(move || {
        caf_core::crash::dismiss_crash_report(&report_id, never_again.unwrap_or(false))
    })
    .await
}

fn do_open_url(url: &str) -> Result<(), CatermError> {
    // Web and mail links only: this must never become a way to launch local files or other
    // protocol handlers from the WebView.
    let lower = url.trim_start().to_ascii_lowercase();
    if !(lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("mailto:"))
    {
        return Err(caf_core::error::ValidationError::Generic(format!(
            "refusing to open non-web URL: {url}"
        ))
        .into());
    }
    #[cfg(target_os = "linux")]
    let res = std::process::Command::new("xdg-open").arg(url).spawn();
    // rundll32 hands the URL straight to the default browser. `cmd /C start` re-parsed it, and
    // an `&` in the query string would split the command (and it flashed a console window).
    #[cfg(target_os = "windows")]
    let res = std::process::Command::new("rundll32")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn();
    #[cfg(target_os = "macos")]
    let res = std::process::Command::new("open").arg(url).spawn();
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    let res: Result<(), std::io::Error> = Ok(());

    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    let result = res.map(|_| ()).map_err(|e| {
        caf_core::error::IoError::Generic(format!("Failed to open URL: {e}")).into()
    });
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    let result = res.map_err(|e| {
        caf_core::error::IoError::Generic(format!("Failed to open URL: {e}")).into()
    });

    result
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), CatermError> {
    run_blocking(move || do_open_url(&url)).await
}

#[tauri::command]
pub async fn ai_execute_step(
    host_id: String,
    command: String,
) -> Result<ai::AiExecutionResult, CatermError> {
    run_blocking(move || ai::execute_plan_step(&host_id, &command)).await
}

#[tauri::command]
pub async fn plan_sync(
    host_id: String,
    local_dir: String,
    remote_dir: String,
    direction: sync::SyncDirection,
) -> Result<sync::SyncPlan, CatermError> {
    run_blocking(move || sync::plan_sync(&host_id, &local_dir, &remote_dir, direction)).await
}

#[tauri::command]
pub async fn execute_sync(
    host_id: String,
    plan: sync::SyncPlan,
    direction: sync::SyncDirection,
) -> Result<sync::SyncStats, CatermError> {
    run_blocking(move || sync::execute_sync(&host_id, &plan, direction)).await
}

#[tauri::command]
pub async fn start_watch(
    host_id: String,
    local_dir: String,
    remote_dir: String,
) -> Result<sync::WatchHandle, CatermError> {
    run_blocking(move || sync::start_watch(&host_id, &local_dir, &remote_dir)).await
}

#[tauri::command]
pub async fn stop_watch(id: String) -> Result<(), CatermError> {
    run_blocking(move || sync::stop_watch(&id)).await
}

#[tauri::command]
pub async fn list_watches() -> Result<Vec<sync::WatchInfo>, CatermError> {
    run_blocking(sync::list_watches).await
}

/// GPU on/off for the WebView. Read by the shell before the window exists, so a change only
/// applies after a restart — the Settings page says so and offers to relaunch.
#[tauri::command]
pub fn get_performance_prefs() -> prefs::PerformancePrefs {
    prefs::load_performance_prefs().unwrap_or_default()
}

#[tauri::command]
pub async fn set_performance_prefs(prefs: prefs::PerformancePrefs) -> Result<(), CatermError> {
    run_blocking(move || prefs::save_performance_prefs(&prefs)).await
}

// ---- CAFramework Pro (caf_core::pro) --------------------------------------------------------

#[tauri::command]
pub async fn pro_status() -> Result<caf_core::pro::ProStatus, CatermError> {
    run_blocking(caf_core::pro::status).await
}

#[tauri::command]
pub async fn pro_server_available() -> Result<bool, CatermError> {
    run_blocking(|| Ok(caf_core::pro::server_available())).await
}

#[tauri::command]
pub async fn pro_register(
    email: String,
    password: String,
    name: String,
    locale: String,
) -> Result<(), CatermError> {
    run_blocking(move || caf_core::pro::register(&email, &password, &name, &locale)).await
}

#[tauri::command]
pub async fn pro_resend_verification(email: String) -> Result<(), CatermError> {
    run_blocking(move || caf_core::pro::resend_verification(&email)).await
}

#[tauri::command]
pub async fn pro_forgot_password(email: String, locale: String) -> Result<(), CatermError> {
    run_blocking(move || caf_core::pro::forgot_password(&email, &locale)).await
}

#[tauri::command]
pub async fn pro_login(
    email: String,
    password: String,
) -> Result<caf_core::pro::ProAccount, CatermError> {
    run_blocking(move || caf_core::pro::login(&email, &password)).await
}

#[tauri::command]
pub async fn pro_commit_pending() -> Result<bool, CatermError> {
    run_blocking(caf_core::pro::commit_pending).await
}

#[tauri::command]
pub async fn pro_sync() -> Result<caf_core::pro::SyncOutcome, CatermError> {
    run_blocking(caf_core::pro::sync).await
}

#[tauri::command]
pub async fn pro_start_trial() -> Result<caf_core::pro::SyncOutcome, CatermError> {
    run_blocking(caf_core::pro::start_trial).await
}

#[tauri::command]
pub async fn pro_account() -> Result<caf_core::pro::AccountDetails, CatermError> {
    run_blocking(caf_core::pro::account_details).await
}

#[tauri::command]
pub async fn pro_revoke_device(device_id: String) -> Result<(), CatermError> {
    run_blocking(move || caf_core::pro::revoke_device(&device_id)).await
}

#[tauri::command]
pub async fn pro_logout() -> Result<(), CatermError> {
    run_blocking(caf_core::pro::logout).await
}

#[tauri::command]
pub async fn pro_team() -> Result<caf_core::pro::ProTeamView, CatermError> {
    run_blocking(caf_core::pro::team).await
}

#[tauri::command]
pub async fn pro_team_invite(
    email: String,
    locale: String,
) -> Result<caf_core::pro::ProTeamView, CatermError> {
    run_blocking(move || caf_core::pro::team_invite(&email, &locale)).await
}

#[tauri::command]
pub async fn pro_team_cancel_invite(
    invite_id: String,
) -> Result<caf_core::pro::ProTeamView, CatermError> {
    run_blocking(move || caf_core::pro::team_cancel_invite(&invite_id)).await
}

#[tauri::command]
pub async fn pro_team_remove_member(
    account_id: String,
) -> Result<caf_core::pro::ProTeamView, CatermError> {
    run_blocking(move || caf_core::pro::team_remove_member(&account_id)).await
}

#[tauri::command]
pub async fn pro_team_accept(
    invitation_id: String,
) -> Result<caf_core::pro::ProTeamView, CatermError> {
    run_blocking(move || caf_core::pro::team_accept(&invitation_id)).await
}

#[tauri::command]
pub async fn pro_team_decline(
    invitation_id: String,
) -> Result<caf_core::pro::ProTeamView, CatermError> {
    run_blocking(move || caf_core::pro::team_decline(&invitation_id)).await
}

#[tauri::command]
pub async fn pro_team_leave() -> Result<caf_core::pro::ProTeamView, CatermError> {
    run_blocking(caf_core::pro::team_leave).await
}

#[tauri::command]
pub async fn submit_feedback(
    rating: i32,
    content: String,
    name: Option<String>,
    profession: Option<String>,
) -> Result<(), CatermError> {
    run_blocking(move || feedback::submit_feedback(rating, &content, name.as_deref(), profession.as_deref())).await
}
