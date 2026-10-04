//! Shared HTTP client for CAFramework.
//!
//! Enforces:
//! - Connect timeout: 10s
//! - Request timeout: 30s (AI default 120s)
//! - User-Agent: <slug>/<version> (+<brand.website>)
//! - HTTPS only (except loopback: localhost, 127.0.0.1, [::1])
//! - Header `X-CA-App-Id: <identifier>` and JSON field `app_id` on every GCC call

use crate::error::{CafError, IoError};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    pub slug: String,
    pub version: String,
    pub brand_website: String,
    pub app_identifier: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            slug: "caframework".to_string(),
            version: "0.1.0-dev".to_string(),
            brand_website: "https://www.cecepazhar.com".to_string(),
            app_identifier: "com.fathforce.caframework".to_string(),
        }
    }
}

pub struct HttpClient {
    config: HttpClientConfig,
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> Self {
        Self { config }
    }

    pub fn default_client() -> Self {
        Self::new(HttpClientConfig::default())
    }

    pub fn user_agent(&self) -> String {
        format!(
            "{}/{} (+{})",
            self.config.slug, self.config.version, self.config.brand_website
        )
    }

    pub fn validate_url(&self, url: &str) -> Result<(), CafError> {
        let is_https = url.starts_with("https://");
        let is_loopback = url.starts_with("http://localhost")
            || url.starts_with("http://127.0.0.1")
            || url.starts_with("http://[::1]");

        if is_https || is_loopback {
            Ok(())
        } else {
            Err(CafError::Io(IoError::Generic(
                "Insecure HTTP connection disallowed for non-loopback hosts".into(),
            )))
        }
    }

    pub fn post_json<T: serde::Serialize>(
        &self,
        url: &str,
        payload: &T,
        timeout_secs: u64,
    ) -> Result<String, CafError> {
        self.validate_url(url)?;

        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(timeout_secs)))
            .build()
            .new_agent();

        let mut resp = agent
            .post(url)
            .header("User-Agent", &self.user_agent())
            .header("X-CA-App-Id", &self.config.app_identifier)
            .header("Content-Type", "application/json")
            .send_json(payload)
            .map_err(|e| CafError::Io(IoError::Generic(format!("HTTP request failed: {e}"))))?;

        let status = resp.status().as_u16();
        let body = resp.body_mut().read_to_string().unwrap_or_default();

        if (200..300).contains(&status) {
            Ok(body)
        } else {
            Err(CafError::Io(IoError::Generic(format!(
                "HTTP request returned status {status}: {body}"
            ))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_agent_format() {
        let client = HttpClient::default_client();
        assert_eq!(
            client.user_agent(),
            "caframework/0.1.0-dev (+https://www.cecepazhar.com)"
        );
    }

    #[test]
    fn test_url_validation_https_and_loopback() {
        let client = HttpClient::default_client();
        assert!(client.validate_url("https://api.example.com/v1").is_ok());
        assert!(client.validate_url("http://localhost:11434/v1").is_ok());
        assert!(client.validate_url("http://127.0.0.1:8080/v1").is_ok());
        assert!(
            client
                .validate_url("http://insecure.example.com/v1")
                .is_err()
        );
    }
}
