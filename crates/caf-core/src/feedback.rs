//! In-app feedback submission. Calls the caframework proxy (`/api/feedback`) which
//! forwards to GCC with the API key — the key is NEVER in this crate or the
//! Tauri binary. See apps/caframework/main.go `handleFeedback`.
//!
//! Deliberately fire-and-forget from the user's perspective: a 2xx means the
//! proxy accepted it, a non-2xx surfaces a human-readable error via `CatermError`.

use crate::error::{CatermError, IoError};

const PROXY_URL: &str = "https://caframework.fathforce.com/api/feedback";

/// Submit user feedback. `rating` must be 1-5; `content` is optional but the
/// proxy will reject an empty string — we enforce it here first so the error
/// message is local and instant.
///
/// This function performs a blocking HTTP request; callers in `caf-app`
/// must spawn it on `run_blocking`.
pub fn submit_feedback(
    rating: i32,
    content: &str,
    name: Option<&str>,
    profession: Option<&str>,
) -> Result<(), CatermError> {
    if !(1..=5).contains(&rating) {
        return Err(CatermError::Io(IoError::Generic(
            "Rating must be between 1 and 5".into(),
        )));
    }
    if content.trim().is_empty() {
        return Err(CatermError::Io(IoError::Generic(
            "Feedback cannot be empty".into(),
        )));
    }

    let mut payload = serde_json::json!({
        "rating": rating,
        "content": content.trim(),
    });

    if let Some(n) = name.map(str::trim).filter(|s| !s.is_empty()) {
        payload["name"] = serde_json::Value::String(n.to_string());
    }
    if let Some(p) = profession.map(str::trim).filter(|s| !s.is_empty()) {
        payload["profession"] = serde_json::Value::String(p.to_string());
    }

    let agent = ureq::Agent::new_with_defaults();
    let mut resp = agent
        .post(PROXY_URL)
        .header("Content-Type", "application/json")
        .send_json(&payload)
        .map_err(|e| {
            CatermError::Io(IoError::Generic(format!("Failed to submit feedback: {e}")))
        })?;

    let status = resp.status().as_u16();
    if !(200..300).contains(&status) {
        let body = resp.body_mut().read_to_string().unwrap_or_default();
        return Err(CatermError::Io(IoError::Generic(format!(
            "Feedback proxy rejected request (HTTP {status}): {body}"
        ))));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_rating() {
        assert!(submit_feedback(0, "bagus", None, None).is_err());
        assert!(submit_feedback(6, "bagus", None, None).is_err());
        assert!(submit_feedback(-1, "bagus", None, None).is_err());
    }

    #[test]
    fn rejects_empty_content() {
        assert!(submit_feedback(5, "", None, None).is_err());
        assert!(submit_feedback(5, "   ", None, None).is_err());
    }
}
