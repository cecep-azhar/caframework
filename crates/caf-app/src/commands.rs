//! Thin Tauri command bindings for CAFramework.

use caf_core::{CafError, ai, backup, crash, feedback, notes, prefs, profiles, vault};

async fn run_blocking<F, R>(f: F) -> Result<R, CafError>
where
    F: FnOnce() -> Result<R, CafError> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f).await.map_err(|e| {
        caf_core::CafError::Io(caf_core::error::IoError::Generic(format!(
            "Task join error: {e}"
        )))
    })?
}

// Window management commands
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

// Vault commands
#[tauri::command]
pub async fn is_vault_initialized() -> Result<bool, CafError> {
    run_blocking(vault::is_vault_initialized).await
}

#[tauri::command]
pub async fn validate_vault_password(password: String) -> Result<bool, CafError> {
    run_blocking(move || vault::validate_password(&password)).await
}

#[tauri::command]
pub async fn lock_vault() -> Result<(), CafError> {
    run_blocking(move || {
        vault::lock();
        Ok(())
    })
    .await
}

#[tauri::command]
pub async fn change_master_password(
    old_password: String,
    new_password: String,
) -> Result<(), CafError> {
    run_blocking(move || vault::change_master_password(&old_password, &new_password)).await
}

#[tauri::command]
pub async fn reset_vault() -> Result<(), CafError> {
    run_blocking(vault::reset_vault).await
}

// Profile commands
#[tauri::command]
pub async fn list_profiles() -> Result<Vec<profiles::ProfileRecord>, CafError> {
    run_blocking(profiles::list_profiles).await
}

#[tauri::command]
pub async fn save_profile(
    input: profiles::ProfileInput,
    caller_profile_id: String,
) -> Result<profiles::ProfileRecord, CafError> {
    run_blocking(move || profiles::save_profile(input, &caller_profile_id)).await
}

#[tauri::command]
pub async fn verify_pin(profile_id: String, pin: String) -> Result<bool, CafError> {
    run_blocking(move || profiles::verify_pin(&profile_id, &pin)).await
}

// Notes sample commands (Vertical slice)
#[tauri::command]
pub async fn list_notes(
    caller_profile_id: String,
    is_owner: bool,
) -> Result<Vec<notes::NoteRecord>, CafError> {
    run_blocking(move || notes::list_notes(&caller_profile_id, is_owner)).await
}

#[tauri::command]
pub async fn save_note(
    input: notes::NoteInput,
    caller_profile_id: String,
) -> Result<notes::NoteRecord, CafError> {
    run_blocking(move || notes::save_note(input, &caller_profile_id)).await
}

#[tauri::command]
pub async fn delete_note(id: String, caller_profile_id: String) -> Result<(), CafError> {
    run_blocking(move || notes::delete_note(&id, &caller_profile_id)).await
}

// AI commands
#[tauri::command]
pub async fn get_ai_settings() -> Result<ai::AiSettings, CafError> {
    run_blocking(ai::get_settings).await
}

#[tauri::command]
pub async fn save_ai_settings(settings: ai::AiSettings) -> Result<(), CafError> {
    run_blocking(move || ai::save_settings(&settings)).await
}

#[tauri::command]
pub async fn set_ai_api_key(api_key: String) -> Result<(), CafError> {
    run_blocking(move || ai::set_ai_api_key(&api_key)).await
}

#[tauri::command]
pub async fn clear_ai_api_key() -> Result<(), CafError> {
    run_blocking(ai::clear_ai_api_key).await
}

#[tauri::command]
pub async fn ai_preview_context(level: ai::PrivacyLevel) -> Result<ai::ContextPayload, CafError> {
    run_blocking(move || ai::preview_context_for_current_session(level)).await
}

#[tauri::command]
pub async fn ai_chat(
    prompt: String,
    privacy_level: Option<ai::PrivacyLevel>,
    consent_given: Option<bool>,
) -> Result<ai::AiChatResponse, CafError> {
    run_blocking(move || {
        let session = caf_core::session::get_current_session().unwrap_or_else(|_| {
            caf_core::session::Session::new("default", "member")
        });
        ai::chat(&session, &prompt, privacy_level, consent_given.unwrap_or(false))
    })
    .await
}

// Feedback & Crash commands
#[tauri::command]
pub async fn submit_feedback(
    rating: i32,
    content: String,
    name: Option<String>,
    profession: Option<String>,
) -> Result<(), CafError> {
    run_blocking(move || {
        feedback::submit_feedback(rating, &content, name.as_deref(), profession.as_deref())
    })
    .await
}

#[tauri::command]
pub async fn get_pending_crash_report() -> Result<Option<crash::ScrubbedCrashReport>, CafError> {
    run_blocking(crash::pending_crash_report).await
}

#[tauri::command]
pub async fn dismiss_crash_report(id: String, never_again: Option<bool>) -> Result<(), CafError> {
    run_blocking(move || crash::dismiss_crash_report(&id, never_again.unwrap_or(false))).await
}

// Backup commands
#[tauri::command]
pub async fn export_encrypted_backup(password: String) -> Result<Vec<u8>, CafError> {
    run_blocking(move || backup::export_backup(&password)).await
}

#[tauri::command]
pub async fn import_encrypted_backup(data: Vec<u8>, password: String) -> Result<(), CafError> {
    run_blocking(move || backup::import_backup(&data, &password)).await
}

// Performance & Preferences
#[tauri::command]
pub async fn get_performance_prefs() -> Result<prefs::PerformancePrefs, CafError> {
    run_blocking(prefs::load_performance_prefs).await
}

#[tauri::command]
pub async fn set_performance_prefs(prefs: prefs::PerformancePrefs) -> Result<(), CafError> {
    run_blocking(move || prefs::save_performance_prefs(&prefs)).await
}
