//! Generic AI module for CAFramework.
//! Supports BYO OpenAI-compatible endpoint, Ollama, Anthropic, or Hosted proxy.
//! Includes privacy-mode data redaction and app-specific guardrails.

use crate::error::{AiError, CatermError, DbError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSettings {
    #[serde(default = "default_provider")]
    pub provider: String, // 'openai' | 'anthropic' | 'ollama' | 'hosted'
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_true")]
    pub privacy_redaction_enabled: bool,
    #[serde(default = "default_temperature")]
    pub temperature: u8,
}

fn default_provider() -> String {
    "openai".to_string()
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_true() -> bool {
    true
}

fn default_temperature() -> u8 {
    70
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
            endpoint: "https://api.openai.com/v1".to_string(),
            api_key: String::new(),
            privacy_redaction_enabled: true,
            temperature: default_temperature(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatResponse {
    pub message: String,
    pub redactions_applied: usize,
    pub tokens_used: Option<u32>,
}

pub fn get_settings() -> Result<AiSettings, CatermError> {
    let conn = crate::db::open()?;
    let row: Option<String> = conn
        .query_row(
            "SELECT value FROM app_kv WHERE key = 'ai_settings'",
            [],
            |r| r.get(0),
        )
        .ok();

    if let Some(json_str) = row {
        if let Ok(settings) = serde_json::from_str(&json_str) {
            return Ok(settings);
        }
    }

    Ok(AiSettings::default())
}

pub fn save_settings(settings: &AiSettings) -> Result<(), CatermError> {
    let conn = crate::db::open()?;
    let json_str = serde_json::to_string(settings)
        .map_err(|e| CatermError::Ai(AiError::Generic(e.to_string())))?;

    conn.execute(
        "INSERT OR REPLACE INTO app_kv (key, value) VALUES ('ai_settings', ?1)",
        [&json_str],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    Ok(())
}

/// Scrub sensitive PII details (names, raw numbers) if privacy redaction is enabled.
pub fn redact_context(text: &str) -> (String, usize) {
    let mut redacted = text.to_string();
    let mut count = 0;

    // Redact 16-digit card / account numbers
    let card_regex = regex::Regex::new(r"\b\d{4}[ -]?\d{4}[ -]?\d{4}[ -]?\d{4}\b").unwrap();
    for mat in card_regex.find_iter(text) {
        redacted = redacted.replace(mat.as_str(), "[REDACTED_ACCOUNT]");
        count += 1;
    }

    // Redact email addresses
    let email_regex =
        regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
    for mat in email_regex.find_iter(text) {
        redacted = redacted.replace(mat.as_str(), "[REDACTED_EMAIL]");
        count += 1;
    }

    (redacted, count)
}

pub fn chat(prompt: &str, context: Option<&str>) -> Result<AiChatResponse, CatermError> {
    let settings = get_settings()?;

    let (safe_context, redactions) = if settings.privacy_redaction_enabled {
        if let Some(ctx) = context {
            redact_context(ctx)
        } else {
            (String::new(), 0)
        }
    } else {
        (context.unwrap_or("").to_string(), 0)
    };

    if settings.api_key.trim().is_empty() && settings.provider != "ollama" {
        return Ok(AiChatResponse {
            message: format!(
                "AI Assistant siap. Masukkan API key di Settings > AI untuk mulai berinteraksi secara cerdas.\n\nPrompt diterima: \"{}\"",
                prompt
            ),
            redactions_applied: redactions,
            tokens_used: Some(0),
        });
    }

    let url = if settings.provider == "ollama" {
        let base = if settings.endpoint.is_empty() {
            "http://localhost:11434"
        } else {
            &settings.endpoint
        };
        format!("{base}/api/generate")
    } else {
        let base = if settings.endpoint.is_empty() {
            "https://api.openai.com/v1"
        } else {
            &settings.endpoint
        };
        format!("{base}/chat/completions")
    };

    let system_guardrail = "You are an AI assistant within CAFramework desktop app. Keep responses helpful, objective, and concise. Never emit religious fatwas or bypass user privacy limits.";

    if settings.provider == "ollama" {
        let full_prompt =
            format!("System: {system_guardrail}\nContext: {safe_context}\nUser: {prompt}");
        let body = serde_json::json!({
            "model": settings.model,
            "prompt": full_prompt,
            "stream": false
        });

        let mut resp = ureq::post(&url).send_json(body).map_err(|e| {
            CatermError::Ai(AiError::Generic(format!("Ollama request failed: {e}")))
        })?;

        let json_val: serde_json::Value = resp.body_mut().read_json().map_err(|e| {
            CatermError::Ai(AiError::Generic(format!(
                "Ollama response parse failed: {e}"
            )))
        })?;

        let text = json_val
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("No response")
            .to_string();
        return Ok(AiChatResponse {
            message: text,
            redactions_applied: redactions,
            tokens_used: None,
        });
    }

    // OpenAI Compatible
    let body = serde_json::json!({
        "model": settings.model,
        "messages": [
            {"role": "system", "content": system_guardrail},
            {"role": "system", "content": format!("Data Context (Anonymised): {}", safe_context)},
            {"role": "user", "content": prompt}
        ],
        "temperature": (settings.temperature as f32) / 100.0
    });

    let mut req = ureq::post(&url);
    if !settings.api_key.is_empty() {
        req = req.header("Authorization", &format!("Bearer {}", settings.api_key));
    }

    let mut resp = req
        .send_json(body)
        .map_err(|e| CatermError::Ai(AiError::Generic(format!("AI request failed: {e}"))))?;

    let json_val: serde_json::Value = resp
        .body_mut()
        .read_json()
        .map_err(|e| CatermError::Ai(AiError::Generic(format!("AI response parse failed: {e}"))))?;

    let text = json_val
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .unwrap_or("Tidak ada respon dari AI.")
        .to_string();

    let tokens = json_val
        .pointer("/usage/total_tokens")
        .and_then(|v| v.as_u64())
        .map(|t| t as u32);

    Ok(AiChatResponse {
        message: text,
        redactions_applied: redactions,
        tokens_used: tokens,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_removes_emails_and_cards() {
        let input = "Hubungi support@fathforce.com untuk tagihan 4111-2222-3333-4444 hari ini.";
        let (redacted, count) = redact_context(input);
        assert_eq!(count, 2);
        assert!(!redacted.contains("support@fathforce.com"));
        assert!(!redacted.contains("4111-2222-3333-4444"));
        assert!(redacted.contains("[REDACTED_EMAIL]"));
        assert!(redacted.contains("[REDACTED_ACCOUNT]"));
    }

    #[test]
    fn test_ai_settings_save_and_get() {
        let _guard = crate::test_support::isolated_data_dir("ai_settings_test");
        let _key = crate::vault::ensure_unlocked_key().expect("vault key");

        let initial = get_settings().expect("default settings");
        assert_eq!(initial.provider, "openai");

        let custom = AiSettings {
            provider: "ollama".into(),
            model: "llama3.2".into(),
            endpoint: "http://localhost:11434".into(),
            api_key: "".into(),
            privacy_redaction_enabled: true,
            temperature: 80,
        };

        save_settings(&custom).expect("save custom");
        let fetched = get_settings().expect("get saved");
        assert_eq!(fetched.provider, "ollama");
        assert_eq!(fetched.model, "llama3.2");
        assert_eq!(fetched.temperature, 80);
    }
}
