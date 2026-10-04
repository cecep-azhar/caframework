//! PII detection and redaction utilities.

use regex::Regex;
use std::sync::OnceLock;

macro_rules! static_regex {
    ($name:ident, $pattern:expr) => {
        fn $name() -> Option<&'static Regex> {
            static CELL: OnceLock<Option<Regex>> = OnceLock::new();
            CELL.get_or_init(|| Regex::new($pattern).ok()).as_ref()
        }
    };
}

// 1. Emails: Using the provided spec instruction to fix [A-Z|a-z] error:
static_regex!(email_re, r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b");

// 2. Payment Cards: 13-19 digits with Luhn check and common separators.
static_regex!(card_re, r"\b(?:\d[ -]*?){13,19}\b");

pub fn is_luhn_valid(number: &str) -> bool {
    let digits: Vec<u32> = number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .filter_map(|c| c.to_digit(10))
        .collect();

    if digits.is_empty() {
        return false;
    }

    let mut sum = 0;
    let mut double = false;

    for &digit in digits.iter().rev() {
        let mut d = digit;
        if double {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        double = !double;
    }

    sum % 10 == 0
}

// 3. Indonesian Mobile Numbers (+62, 62, 08, 9-13 digits)
static_regex!(id_phone_re, r"(\+62|62|08)[0-9]{7,11}\b");

// 4. NIK: 16 digits (plausible province/date format logic check can be added if needed, but regex 16 digits is base)
static_regex!(id_nik_re, r"\b\d{16}\b");

// But let's refine NIK a bit to avoid matching valid cards if possible or generic 16 digits that are cards,
// however we will handle order in redact.
// A NIK structure is: XX.XX.XX.DDMMYY.XXXX, where XX is prov/city/kec (so not all 0s). Let's use basic 16 digits first.

// 5. NPWP: 15/16 digits, dotted or plain.
static_regex!(
    id_npwp_re,
    r"\b(?:\d{2}[.-]?\d{3}[.-]?\d{3}[.-]?\d{1}[.-]?\d{3}[.-]?\d{3}|\d{15,16})\b"
);

// 6. IBAN
static_regex!(iban_re, r"\b[A-Z]{2}\d{2}[A-Z0-9]{1,30}\b");

// Old ones from crash.rs
static_regex!(
    private_key_re,
    r"(?s)-----BEGIN [A-Z ]+PRIVATE KEY-----.*?-----END [A-Z ]+PRIVATE KEY-----"
);
static_regex!(api_key_re, r"\bsk-[a-zA-Z0-9]{20,}|\bgsk_[a-zA-Z0-9]{20,}");
static_regex!(
    secret_kv_re,
    r#"(?i)("?(?:password|passphrase|secret|token|api_key|auth)"?\s*[:=]\s*)"?([^"'\s,}]+)"?"#
);
static_regex!(
    ipv4_re,
    r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b"
);

#[derive(Default)]
pub struct PiiScrubber {
    home_dir: String,
    username: String,
}

impl PiiScrubber {
    pub fn new() -> Self {
        let home_dir = directories::BaseDirs::new()
            .map(|b| b.home_dir().to_string_lossy().into_owned())
            .unwrap_or_default();
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_default();
        Self { home_dir, username }
    }

    pub fn scrub_text(&self, input: &str) -> (String, usize) {
        let mut text = input.to_string();
        let mut redactions_applied = 0;

        if let Some(re) = private_key_re() {
            text = re
                .replace_all(&text, |_: &regex::Captures| {
                    redactions_applied += 1;
                    "[REDACTED_PRIVATE_KEY]".to_string()
                })
                .to_string();
        }
        if let Some(re) = api_key_re() {
            text = re
                .replace_all(&text, |_: &regex::Captures| {
                    redactions_applied += 1;
                    "[REDACTED_API_KEY]".to_string()
                })
                .to_string();
        }
        if let Some(re) = secret_kv_re() {
            text = re
                .replace_all(&text, |caps: &regex::Captures| {
                    redactions_applied += 1;
                    format!("{}[REDACTED]", &caps[1])
                })
                .to_string();
        }
        if let Some(re) = ipv4_re() {
            text = re
                .replace_all(&text, |caps: &regex::Captures| match &caps[0] {
                    "127.0.0.1" | "0.0.0.0" => caps[0].to_string(),
                    _ => {
                        redactions_applied += 1;
                        "[REDACTED_IP]".to_string()
                    }
                })
                .to_string();
        }

        // Email
        if let Some(re) = email_re() {
            text = re
                .replace_all(&text, |_: &regex::Captures| {
                    redactions_applied += 1;
                    "[REDACTED_EMAIL]".to_string()
                })
                .to_string();
        }

        // Card (with Luhn)
        if let Some(re) = card_re() {
            text = re
                .replace_all(&text, |caps: &regex::Captures| {
                    let s = &caps[0];
                    if is_luhn_valid(s) {
                        redactions_applied += 1;
                        "[REDACTED_CARD]".to_string()
                    } else {
                        s.to_string()
                    }
                })
                .to_string();
        }

        // NIK (16 digits specifically, checking if not all 0s, etc if possible, but minimal requirement is 16 digits)
        if let Some(re) = id_nik_re() {
            text = re
                .replace_all(&text, |caps: &regex::Captures| {
                    // Basic check, could refine. Let's check it's not a card (already handled if we order correctly,
                    // but card_re only matches if luhn valid, so what if it's 16 digits non-luhn?
                    // The prompt says: "a 16-digit non-Luhn order number is **not** redacted as a card".
                    // Wait, should it be redacted as a NIK? We might need some province validation.
                    // Let's add basic province validation for NIK. (Province codes in Indo are 11-94).
                    let s = &caps[0];
                    let prov: u32 = s[0..2].parse().unwrap_or(0);
                    if (11..=94).contains(&prov) {
                        redactions_applied += 1;
                        "[REDACTED_NIK]".to_string()
                    } else {
                        s.to_string() // e.g. order number like 0000000000000000
                    }
                })
                .to_string();
        }

        // NPWP
        // Let's make sure it really looks like NPWP (15 or 16 digits)
        if let Some(re) = id_npwp_re() {
            text = re
                .replace_all(&text, |caps: &regex::Captures| {
                    // Check if it's dotted
                    let s = &caps[0];
                    if s.contains('.') {
                        redactions_applied += 1;
                        "[REDACTED_NPWP]".to_string()
                    } else {
                        // Need more check for plain to differentiate from NIK. NPWP is 15 or 16 digits.
                        // But let's keep it simple as per prompt. The dotted is most common anyway.
                        // If it's plain 15 digits it won't match NIK.
                        if s.len() == 15 {
                            redactions_applied += 1;
                            "[REDACTED_NPWP]".to_string()
                        } else {
                            // 16 digits NPWP, could overlap with NIK. That's fine.
                            redactions_applied += 1;
                            "[REDACTED_NPWP]".to_string()
                        }
                    }
                })
                .to_string();
        }

        // Phone
        if let Some(re) = id_phone_re() {
            text = re
                .replace_all(&text, |_: &regex::Captures| {
                    redactions_applied += 1;
                    "[REDACTED_PHONE]".to_string()
                })
                .to_string();
        }

        // IBAN
        if let Some(re) = iban_re() {
            text = re
                .replace_all(&text, |_: &regex::Captures| {
                    redactions_applied += 1;
                    "[REDACTED_IBAN]".to_string()
                })
                .to_string();
        }

        if !self.home_dir.is_empty() && text.contains(&self.home_dir) {
            text = text.replace(&self.home_dir, "~");
            // Don't count home dir as one of the new main redactions maybe?
            // Actually crash.rs didn't count it. We'll count it.
            redactions_applied += 1;
        }
        if self.username.len() > 2 && text.contains(&self.username) {
            text = text.replace(&self.username, "[USER]");
            redactions_applied += 1;
        }

        (text, redactions_applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_redaction() {
        let scrubber = PiiScrubber::default();
        let (res, count) = scrubber.scrub_text("Contact user@example.com for help");
        assert_eq!(res, "Contact [REDACTED_EMAIL] for help");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_luhn_algorithm() {
        assert!(is_luhn_valid("49927398716"));
        assert!(is_luhn_valid("4111111111111111"));
        assert!(!is_luhn_valid("49927398717"));
        assert!(!is_luhn_valid("1234567812345678"));
    }

    #[test]
    fn test_card_redaction_luhn_valid() {
        let scrubber = PiiScrubber::default();
        let (res, count) = scrubber.scrub_text("Card number: 4111 1111 1111 1111");
        assert_eq!(res, "Card number: [REDACTED_CARD]");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_card_non_luhn_not_redacted() {
        let scrubber = PiiScrubber::default();
        let (res, _count) = scrubber.scrub_text("Order ID: 1234567812345678");
        // Non-luhn 16 digits should not be redacted as card
        assert!(!res.contains("[REDACTED_CARD]"));
    }

    #[test]
    fn test_indonesian_phone_redaction() {
        let scrubber = PiiScrubber::default();
        let (res1, c1) = scrubber.scrub_text("Call +6281234567890 now");
        assert_eq!(res1, "Call [REDACTED_PHONE] now");
        assert_eq!(c1, 1);

        let (res2, c2) = scrubber.scrub_text("Call 085220696117 now");
        assert_eq!(res2, "Call [REDACTED_PHONE] now");
        assert_eq!(c2, 1);
    }

    #[test]
    fn test_api_keys_and_tokens() {
        let scrubber = PiiScrubber::default();
        let (res, count) = scrubber.scrub_text("My key is sk-123456789012345678901234567890");
        assert_eq!(res, "My key is [REDACTED_API_KEY]");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_ipv4_redaction() {
        let scrubber = PiiScrubber::default();
        let (res, count) = scrubber.scrub_text("Server at 192.168.1.50 and 127.0.0.1");
        assert_eq!(res, "Server at [REDACTED_IP] and 127.0.0.1");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_iban_redaction() {
        let scrubber = PiiScrubber::default();
        let (res, count) = scrubber.scrub_text("Transfer to DE89370400440532013000");
        assert_eq!(res, "Transfer to [REDACTED_IBAN]");
        assert_eq!(count, 1);
    }
}
