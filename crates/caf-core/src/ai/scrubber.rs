//! Privacy redactor & secret scrubber for AI prompts and terminal output.
//! Redacts private SSH keys, bearer tokens, JWTs, AWS credentials, CLI passwords, and ANSI codes.

use regex::Regex;
use std::sync::LazyLock;

static RE_PRIVATE_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)-----BEGIN [A-Z0-9_-]+ PRIVATE KEY-----.*?-----END [A-Z0-9_-]+ PRIVATE KEY-----",
    )
    .unwrap_or_else(|_| Regex::new("$^").expect("fallback regex"))
});

static RE_BEARER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\bBearer\s+[A-Za-z0-9\-._~+/]+=*")
        .unwrap_or_else(|_| Regex::new("$^").expect("fallback regex"))
});

static RE_JWT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b")
        .unwrap_or_else(|_| Regex::new("$^").expect("fallback regex"))
});

static RE_AWS_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(AKIA|ABIA|ACCA|ASIA)[0-9A-Z]{16}\b")
        .unwrap_or_else(|_| Regex::new("$^").expect("fallback regex"))
});

static RE_OPENAI_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bsk-[a-zA-Z0-9_-]{20,}\b")
        .unwrap_or_else(|_| Regex::new("$^").expect("fallback regex"))
});

static RE_GITHUB_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36}\b")
        .unwrap_or_else(|_| Regex::new("$^").expect("fallback regex"))
});

static RE_PASSWORD_FLAG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)(--password[=\s]+|--pass[=\s]+|--secret[=\s]+|-p\s+)(['"]?[^\s'"]+['"]?)"#)
        .unwrap_or_else(|_| Regex::new("$^").expect("fallback regex"))
});

static RE_URL_BASIC_AUTH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https?://[^:\s]+:[^@\s]+@"#)
        .unwrap_or_else(|_| Regex::new("$^").expect("fallback regex"))
});

static RE_ANSI_ESCAPE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\x1b\[[0-9;]*[a-zA-Z]|\x1b\][^\x07\x1b]*(\x07|\x1b\\)"#)
        .unwrap_or_else(|_| Regex::new("$^").expect("fallback regex"))
});

#[derive(Debug, Clone, Default)]
pub struct PromptScrubber;

impl PromptScrubber {
    /// # Infallible: compiles static regex rules on demand
    pub fn new() -> Self {
        Self
    }

    /// Redact sensitive secrets, keys, credentials, and ANSI codes from text.
    /// # Infallible: in-memory regex scrubbing
    pub fn scrub(&self, input: &str) -> String {
        let mut text = RE_ANSI_ESCAPE.replace_all(input, "").into_owned();

        if RE_PRIVATE_KEY.is_match(&text) {
            text = RE_PRIVATE_KEY
                .replace_all(&text, "[REDACTED_PRIVATE_KEY]")
                .into_owned();
        }

        if RE_BEARER.is_match(&text) {
            text = RE_BEARER
                .replace_all(&text, "Bearer [REDACTED_BEARER_TOKEN]")
                .into_owned();
        }

        if RE_JWT.is_match(&text) {
            text = RE_JWT
                .replace_all(&text, "[REDACTED_JWT_TOKEN]")
                .into_owned();
        }

        if RE_AWS_KEY.is_match(&text) {
            text = RE_AWS_KEY
                .replace_all(&text, "[REDACTED_AWS_KEY]")
                .into_owned();
        }

        if RE_OPENAI_KEY.is_match(&text) {
            text = RE_OPENAI_KEY
                .replace_all(&text, "[REDACTED_API_KEY]")
                .into_owned();
        }

        if RE_GITHUB_KEY.is_match(&text) {
            text = RE_GITHUB_KEY
                .replace_all(&text, "[REDACTED_GITHUB_TOKEN]")
                .into_owned();
        }

        if RE_PASSWORD_FLAG.is_match(&text) {
            text = RE_PASSWORD_FLAG
                .replace_all(&text, "$1[REDACTED_PASSWORD]")
                .into_owned();
        }

        if RE_URL_BASIC_AUTH.is_match(&text) {
            text = RE_URL_BASIC_AUTH
                .replace_all(&text, "https://[REDACTED_AUTH]@")
                .into_owned();
        }

        text
    }

    /// Scrub multiple terminal lines, stripping empty or trailing ANSI artifacts.
    /// # Infallible: in-memory string scrubbing across lines
    pub fn scrub_lines(&self, lines: &[String]) -> Vec<String> {
        lines.iter().map(|line| self.scrub(line)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrub_bearer_token() {
        let scrubber = PromptScrubber::new();
        let raw = "curl -H \"Authorization: Bearer secret_token_123\" https://api.com";
        let scrubbed = scrubber.scrub(raw);
        assert!(scrubbed.contains("[REDACTED_BEARER_TOKEN]"));
        assert!(!scrubbed.contains("secret_token_123"));
    }

    #[test]
    fn test_scrub_password_flag() {
        let scrubber = PromptScrubber::new();
        let raw = "mysql -u root --password=supersecret -h 127.0.0.1";
        let scrubbed = scrubber.scrub(raw);
        assert!(scrubbed.contains("[REDACTED_PASSWORD]"));
        assert!(!scrubbed.contains("supersecret"));
    }

    #[test]
    fn test_scrub_private_key() {
        let scrubber = PromptScrubber::new();
        let raw = "Key:\n-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA0...\n-----END RSA PRIVATE KEY-----\nDone";
        let scrubbed = scrubber.scrub(raw);
        assert!(scrubbed.contains("[REDACTED_PRIVATE_KEY]"));
        assert!(!scrubbed.contains("MIIEowIBAAKCAQEA0"));
    }

    #[test]
    fn test_scrub_ansi_codes() {
        let scrubber = PromptScrubber::new();
        let raw = "\x1b[31mError:\x1b[0m Connection failed";
        let scrubbed = scrubber.scrub(raw);
        assert_eq!(scrubbed, "Error: Connection failed");
    }
}
