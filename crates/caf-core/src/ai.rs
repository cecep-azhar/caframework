//! Generic AI module for CAFramework (FR-7).
//! Supports Off, BYO (OpenAI-compatible endpoints / Ollama / LM Studio), and Hosted modes.
//! Provides privacy level scoping (Summary, Detailed, Full), context scrubbing, compiled guardrails,
//! and secure API key management (keys are stored in vault and never returned to frontend).

use crate::error::{AiError, CatermError, DbError};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const GUARDRAILS_TEXT: &str = include_str!("ai/guardrails.txt");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum AiMode {
    #[default]
    Off,
    Byo,
    Hosted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum PrivacyLevel {
    #[default]
    Summary,
    Detailed,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AiSettings {
    #[serde(default)]
    pub mode: AiMode,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: u8, // 0 - 200 (representing 0.0 - 2.0)
    #[serde(default = "default_response_language")]
    pub response_language: String, // "auto" | "id" | "en"
    #[serde(default = "default_tone")]
    pub tone: String, // "neutral" | "friendly" | "formal" | "concise"
    #[serde(default)]
    pub custom_instructions: String, // <= 2000 chars
    #[serde(default)]
    pub privacy_level_default: PrivacyLevel,
    #[serde(default)]
    pub allow_full_detail_for_local: bool,
    #[serde(default)]
    pub has_api_key: bool,
}

fn default_base_url() -> String {
    "http://localhost:11434/v1".to_string()
}

fn default_model() -> String {
    "llama3.2".to_string()
}

fn default_temperature() -> u8 {
    70
}

fn default_response_language() -> String {
    "auto".to_string()
}

fn default_tone() -> String {
    "neutral".to_string()
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            mode: AiMode::Off,
            base_url: default_base_url(),
            model: default_model(),
            temperature: default_temperature(),
            response_language: default_response_language(),
            tone: default_tone(),
            custom_instructions: String::new(),
            privacy_level_default: PrivacyLevel::Summary,
            allow_full_detail_for_local: false,
            has_api_key: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAiConfig {
    pub mode: AiMode,
    pub base_url: String,
    pub model: String,
    pub temperature: u8,
    pub response_language: String,
    pub tone: String,
    pub custom_instructions: String,
    pub privacy_level_default: PrivacyLevel,
    pub allow_full_detail_for_local: bool,
    pub api_key: String,
}

impl From<StoredAiConfig> for AiSettings {
    fn from(c: StoredAiConfig) -> Self {
        Self {
            mode: c.mode,
            base_url: c.base_url,
            model: c.model,
            temperature: c.temperature,
            response_language: c.response_language,
            tone: c.tone,
            custom_instructions: c.custom_instructions,
            privacy_level_default: c.privacy_level_default,
            allow_full_detail_for_local: c.allow_full_detail_for_local,
            has_api_key: !c.api_key.trim().is_empty(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AiChatResponse {
    pub message: String,
    pub redactions_applied: usize,
    pub tokens_used: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ContextPayload {
    pub level: PrivacyLevel,
    pub content: String,
    pub redacted_count: usize,
}

fn get_stored_config() -> Result<StoredAiConfig, CatermError> {
    let conn = crate::db::open()?;
    let row: Option<String> = conn
        .query_row(
            "SELECT value FROM app_kv WHERE key = 'ai_settings'",
            [],
            |r| r.get(0),
        )
        .ok();

    if let Some(json_str) = row
        && let Ok(cfg) = serde_json::from_str::<StoredAiConfig>(&json_str)
    {
        return Ok(cfg);
    }

    Ok(StoredAiConfig {
        mode: AiMode::Off,
        base_url: default_base_url(),
        model: default_model(),
        temperature: default_temperature(),
        response_language: default_response_language(),
        tone: default_tone(),
        custom_instructions: String::new(),
        privacy_level_default: PrivacyLevel::Summary,
        allow_full_detail_for_local: false,
        api_key: String::new(),
    })
}

fn save_stored_config(cfg: &StoredAiConfig) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    let json_str =
        serde_json::to_string(cfg).map_err(|e| CatermError::Ai(AiError::Generic(e.to_string())))?;

    conn.execute(
        "INSERT OR REPLACE INTO app_kv (key, value) VALUES ('ai_settings', ?1)",
        [&json_str],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    Ok(())
}

/// Helper to parse host from base_url without pulling external crates
fn parse_host(url_str: &str) -> Option<String> {
    let trimmed = url_str.trim();
    let after_scheme = if let Some(idx) = trimmed.find("://") {
        &trimmed[idx + 3..]
    } else {
        trimmed
    };
    let host_port = after_scheme.split('/').next().unwrap_or("");
    let host = host_port.split(':').next().unwrap_or("").to_lowercase();
    if host.is_empty() { None } else { Some(host) }
}

pub fn is_loopback(url_str: &str) -> bool {
    if let Some(host) = parse_host(url_str) {
        host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "[::1]"
    } else {
        false
    }
}

pub fn get_settings() -> Result<AiSettings, CatermError> {
    let cfg = get_stored_config()?;
    Ok(cfg.into())
}

pub fn save_settings(settings: &AiSettings) -> Result<(), CatermError> {
    // Validation
    if settings.temperature > 200 {
        return Err(CatermError::Validation(
            crate::error::ValidationError::InvalidFormat,
        ));
    }
    if settings.custom_instructions.len() > 2000 {
        return Err(CatermError::Validation(
            crate::error::ValidationError::InvalidFormat,
        ));
    }

    let mut current = get_stored_config()?;

    // If base_url changed to a different host, clear the stored api key
    let old_host = parse_host(&current.base_url);
    let new_host = parse_host(&settings.base_url);
    if old_host != new_host {
        current.api_key.clear();
    }

    current.mode = settings.mode.clone();
    current.base_url = settings.base_url.clone();
    current.model = settings.model.clone();
    current.temperature = settings.temperature;
    current.response_language = settings.response_language.clone();
    current.tone = settings.tone.clone();
    current.custom_instructions = settings.custom_instructions.clone();
    current.privacy_level_default = settings.privacy_level_default.clone();
    current.allow_full_detail_for_local = settings.allow_full_detail_for_local;

    save_stored_config(&current)?;
    Ok(())
}

pub fn set_ai_api_key(key: &str) -> Result<(), CatermError> {
    let mut current = get_stored_config()?;
    current.api_key = key.trim().to_string();
    save_stored_config(&current)?;
    Ok(())
}

pub fn clear_ai_api_key() -> Result<(), CatermError> {
    let mut current = get_stored_config()?;
    current.api_key.clear();
    save_stored_config(&current)?;
    Ok(())
}

/// Compose the AI system prompt following the strict order:
/// Guardrails -> Framework format rules -> User custom instructions -> Context -> User message
pub fn compose_prompt(
    guardrails: &str,
    tone: &str,
    response_language: &str,
    custom_instructions: &str,
    context: &str,
    user_prompt: &str,
) -> (String, String) {
    let mut system_builder = String::new();

    // 1. Guardrail
    system_builder.push_str(guardrails.trim());
    system_builder.push_str("\n\n");

    // 2. Framework format rules
    system_builder.push_str("Framework Format Rules:\n");
    system_builder.push_str(&format!("- Tone: {}\n", tone));
    system_builder.push_str(&format!("- Response Language: {}\n", response_language));
    system_builder.push_str("- Format responses cleanly using markdown when appropriate.\n\n");

    // 3. User custom instructions
    if !custom_instructions.trim().is_empty() {
        system_builder.push_str("User Custom Instructions:\n");
        system_builder.push_str(custom_instructions.trim());
        system_builder.push_str("\n\n");
    }

    // 4. Context
    if !context.trim().is_empty() {
        system_builder.push_str("Context Data (Scrubbed):\n");
        system_builder.push_str(context.trim());
        system_builder.push_str("\n\n");
    }

    (system_builder.trim().to_string(), user_prompt.to_string())
}

/// Scrub PII using the central PII scrubber from pii.rs
pub fn scrub_text(text: &str) -> (String, usize) {
    let scrubber = crate::pii::PiiScrubber::new();
    scrubber.scrub_text(text)
}

/// Preview context for current session with given privacy level
pub fn preview_context_for_current_session(level: PrivacyLevel) -> Result<ContextPayload, CatermError> {
    let session = crate::session::get_current_session().unwrap_or_else(|_| {
        crate::session::Session::new("default", "member")
    });
    let cfg = get_stored_config()?;
    build_context(
        &session,
        &level,
        true, // consent is implied when user requests a preview
        cfg.allow_full_detail_for_local,
        &cfg.base_url,
    )
}

/// Build context for a given privacy level
pub fn build_context(
    session: &crate::session::Session,
    level: &PrivacyLevel,
    consent_given: bool,
    allow_full_local: bool,
    base_url: &str,
) -> Result<ContextPayload, CatermError> {
    match level {
        PrivacyLevel::Summary => {
            // Aggregate only: profile count, enabled modules, note count
            let conn = crate::db::open()?;
            let note_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))
                .unwrap_or(0);
            let profile_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM profiles", [], |r| r.get(0))
                .unwrap_or(1);

            let content = format!(
                "Application Summary:\n- Active Profiles: {}\n- Total Notes: {}\n- Modules: Notes, AI",
                profile_count, note_count
            );
            let (scrubbed, count) = scrub_text(&content);
            Ok(ContextPayload {
                level: PrivacyLevel::Summary,
                content: scrubbed,
                redacted_count: count,
            })
        }
        PrivacyLevel::Detailed => {
            if !consent_given {
                return Err(CatermError::Ai(AiError::ConsentRequired));
            }

            let is_super = session.role == "super_admin";
            let scope = crate::visibility::scope(&session.profile_id, is_super);
            let conn = crate::db::open()?;
            let query = format!(
                "SELECT title, content FROM notes WHERE {} ORDER BY updated_at DESC LIMIT 10",
                scope.sql_clause
            );
            let mut stmt = conn
                .prepare(&query)
                .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
            let rows = stmt
                .query_map([], |r| {
                    let title: String = r.get(0)?;
                    let content: String = r.get(1)?;
                    Ok(format!("Note: {}\n{}", title, content))
                })
                .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

            let mut notes_text = String::new();
            for r in rows {
                if let Ok(entry) = r {
                    notes_text.push_str(&entry);
                    notes_text.push_str("\n\n");
                }
            }

            let (scrubbed, count) = scrub_text(&notes_text);
            Ok(ContextPayload {
                level: PrivacyLevel::Detailed,
                content: scrubbed,
                redacted_count: count,
            })
        }
        PrivacyLevel::Full => {
            if !allow_full_local || !is_loopback(base_url) {
                return Err(CatermError::Ai(AiError::Generic(
                    "Full privacy level only allowed for local loopback endpoints with allow_full_detail_for_local enabled".to_string(),
                )));
            }

            let is_super = session.role == "super_admin";
            let scope = crate::visibility::scope(&session.profile_id, is_super);
            let conn = crate::db::open()?;
            let query = format!(
                "SELECT title, content FROM notes WHERE {} ORDER BY updated_at DESC LIMIT 20",
                scope.sql_clause
            );
            let mut stmt = conn
                .prepare(&query)
                .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;
            let rows = stmt
                .query_map([], |r| {
                    let title: String = r.get(0)?;
                    let content: String = r.get(1)?;
                    Ok(format!("Note: {}\n{}", title, content))
                })
                .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

            let mut notes_text = String::new();
            for r in rows {
                if let Ok(entry) = r {
                    notes_text.push_str(&entry);
                    notes_text.push_str("\n\n");
                }
            }

            // In Full mode, still run basic scrubber for high sensitivity tokens
            let (scrubbed, count) = scrub_text(&notes_text);
            Ok(ContextPayload {
                level: PrivacyLevel::Full,
                content: scrubbed,
                redacted_count: count,
            })
        }
    }
}

pub fn chat(
    session: &crate::session::Session,
    prompt: &str,
    privacy_level: Option<PrivacyLevel>,
    consent_given: bool,
) -> Result<AiChatResponse, CatermError> {
    let cfg = get_stored_config()?;

    if cfg.mode == AiMode::Off {
        return Err(CatermError::Ai(AiError::Disabled));
    }

    if cfg.mode == AiMode::Hosted {
        // Hosted requires modules.pro & valid license
        let pro_status = crate::pro::get_pro_status()?;
        if !pro_status.is_pro {
            return Err(CatermError::Ai(AiError::QuotaExceeded));
        }
    }

    let level = privacy_level.unwrap_or(cfg.privacy_level_default.clone());
    let context_payload = build_context(
        session,
        &level,
        consent_given,
        cfg.allow_full_detail_for_local,
        &cfg.base_url,
    )?;

    // Scrub user prompt
    let (scrubbed_prompt, prompt_redactions) = scrub_text(prompt);
    let total_redactions = context_payload.redacted_count + prompt_redactions;

    let (system_msg, user_msg) = compose_prompt(
        GUARDRAILS_TEXT,
        &cfg.tone,
        &cfg.response_language,
        &cfg.custom_instructions,
        &context_payload.content,
        &scrubbed_prompt,
    );

    // Call OpenAI-compatible completions endpoint
    let url = format!("{}/chat/completions", cfg.base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": cfg.model,
        "messages": [
            {"role": "system", "content": system_msg},
            {"role": "user", "content": user_msg}
        ],
        "temperature": (cfg.temperature as f32) / 100.0
    });

    let mut req = ureq::post(&url);
    if !cfg.api_key.is_empty() {
        req = req.header("Authorization", &format!("Bearer {}", cfg.api_key));
    }

    let mut resp = req.send_json(body).map_err(|e| match e {
        ureq::Error::StatusCode(401) | ureq::Error::StatusCode(403) => {
            CatermError::Ai(AiError::ApiKeyMissing)
        }
        ureq::Error::StatusCode(402) | ureq::Error::StatusCode(429) => {
            CatermError::Ai(AiError::QuotaExceeded)
        }
        ureq::Error::StatusCode(_) => CatermError::Ai(AiError::ProviderUnavailable),
        _ => CatermError::Ai(AiError::Timeout),
    })?;

    let json_val: serde_json::Value = resp
        .body_mut()
        .read_json()
        .map_err(|_| CatermError::Ai(AiError::ProviderUnavailable))?;

    let text = json_val
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let tokens = json_val
        .pointer("/usage/total_tokens")
        .and_then(|v| v.as_u64())
        .map(|t| t as u32);

    Ok(AiChatResponse {
        message: text,
        redactions_applied: total_redactions,
        tokens_used: tokens,
    })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_ai_settings_save_and_get() {
        let _guard = crate::test_support::isolated_data_dir("ai_settings_test");
        let _key = crate::vault::ensure_unlocked_key().expect("vault key");

        let initial = get_settings().expect("default settings");
        assert_eq!(initial.mode, AiMode::Off);
        assert!(!initial.has_api_key);

        let mut custom = initial;
        custom.mode = AiMode::Byo;
        custom.base_url = "http://localhost:11434/v1".into();
        custom.model = "llama3.2".into();
        custom.temperature = 80;
        custom.custom_instructions = "Be concise".into();

        save_settings(&custom).expect("save custom");
        set_ai_api_key("sk-test-secret-12345").expect("set key");

        let fetched = get_settings().expect("get saved");
        assert_eq!(fetched.mode, AiMode::Byo);
        assert_eq!(fetched.model, "llama3.2");
        assert_eq!(fetched.temperature, 80);
        assert!(fetched.has_api_key);

        // Switching base_url to another host should clear the stored api key
        let mut switched = fetched.clone();
        switched.base_url = "https://api.openai.com/v1".into();
        save_settings(&switched).expect("save switched");

        let fetched_after_switch = get_settings().expect("get after switch");
        assert!(!fetched_after_switch.has_api_key);
    }

    #[test]
    fn test_compose_prompt_order() {
        let guardrails = "Strict Guardrails: No religion.";
        let (sys, user) = compose_prompt(
            guardrails,
            "formal",
            "en",
            "Be precise.",
            "Summary Context",
            "Hello AI",
        );

        assert!(sys.starts_with("Strict Guardrails: No religion."));
        assert!(sys.contains("Framework Format Rules:\n- Tone: formal\n- Response Language: en"));
        assert!(sys.contains("User Custom Instructions:\nBe precise."));
        assert!(sys.contains("Context Data (Scrubbed):\nSummary Context"));
        assert_eq!(user, "Hello AI");
    }

    #[test]
    fn test_privacy_levels_context() {
        let _guard = crate::test_support::isolated_data_dir("ai_context_test");
        let _key = crate::vault::ensure_unlocked_key().expect("vault key");

        let session = crate::session::Session::new("prof-1", "member");

        // Summary level should not include names or note texts
        let summary = build_context(
            &session,
            &PrivacyLevel::Summary,
            false,
            false,
            "http://localhost:11434",
        )
        .expect("summary context");
        assert!(!summary.content.contains("Budi Santoso"));
        assert!(!summary.content.contains("Siti Aminah"));
        assert!(summary.content.contains("Application Summary:"));

        // Detailed level without consent fails
        let detailed_err = build_context(
            &session,
            &PrivacyLevel::Detailed,
            false,
            false,
            "http://localhost:11434",
        );
        assert!(matches!(
            detailed_err,
            Err(CatermError::Ai(AiError::ConsentRequired))
        ));

        // Full level on non-loopback fails
        let full_err = build_context(
            &session,
            &PrivacyLevel::Full,
            true,
            true,
            "https://api.openai.com/v1",
        );
        assert!(full_err.is_err());
    }

    #[test]
    fn test_mode_off_returns_disabled() {
        let _guard = crate::test_support::isolated_data_dir("ai_mode_off_test");
        let _key = crate::vault::ensure_unlocked_key().expect("vault key");

        let session = crate::session::Session::new("prof-1", "member");

        let res = chat(&session, "ping", None, false);
        assert!(matches!(res, Err(CatermError::Ai(AiError::Disabled))));
    }
}
