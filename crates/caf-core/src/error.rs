//! `CatermError` taxonomy (REQ-04). `code()` is a public contract: once shipped, a code's
//! *meaning* must never change between versions, even though new codes get added constantly.
//!
//! Fase 0 has no domain logic yet, so every per-domain sub-enum below carries exactly one
//! `Generic(String)` variant reserved as `CAT-<DOMAIN>-000` — the permanent "unclassified"
//! fallback for that domain. Real, numbered errors (`CAT-VAULT-002`, `CAT-AI-005`, ...) get
//! added to these enums by the phase that actually builds that domain (Fase 1 for vault/db,
//! Fase 2 for ssh, Fase 6 for ai, ...), never guessed ahead of time here.

use serde::Serialize;
use thiserror::Error;

macro_rules! domain_error {
    ($name:ident, $domain:literal) => {
        #[derive(Debug, Error)]
        pub enum $name {
            #[error("{0}")]
            Generic(String),
        }

        impl $name {
            pub fn code(&self) -> &'static str {
                match self {
                    Self::Generic(_) => concat!("CAT-", $domain, "-000"),
                }
            }
        }
    };
}

domain_error!(VaultError, "VAULT");
domain_error!(DbError, "DB");
domain_error!(SshError, "SSH");
domain_error!(SftpError, "SFTP");
domain_error!(TunnelError, "TUNNEL");
domain_error!(AiError, "AI");
domain_error!(SyncError, "SYNC");
domain_error!(IoError, "IO");
domain_error!(ValidationError, "VALIDATION");
domain_error!(FtpError, "FTP");
// CAFramework Pro account/license. The message is the server's stable error code (e.g.
// `INVALID_CREDENTIALS`) so the UI can translate it; see crates/caf-core/src/pro.
domain_error!(ProError, "PRO");

#[derive(Debug, Error)]
pub enum CatermError {
    #[error("vault: {0}")]
    Vault(#[from] VaultError),
    #[error("database: {0}")]
    Db(#[from] DbError),
    #[error("ssh: {0}")]
    Ssh(#[from] SshError),
    #[error("sftp: {0}")]
    Sftp(#[from] SftpError),
    #[error("tunnel: {0}")]
    Tunnel(#[from] TunnelError),
    #[error("ai: {0}")]
    Ai(#[from] AiError),
    #[error("sync: {0}")]
    Sync(#[from] SyncError),
    #[error("io: {0}")]
    Io(#[from] IoError),
    #[error("validation: {0}")]
    Validation(#[from] ValidationError),
    #[error("ftp: {0}")]
    Ftp(#[from] FtpError),
    #[error("pro: {0}")]
    Pro(#[from] ProError),
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

impl CatermError {
    /// Stable public code, `CAT-<DOMAIN>-<NNN>`. Never re-purpose an existing code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Vault(e) => e.code(),
            Self::Db(e) => e.code(),
            Self::Ssh(e) => e.code(),
            Self::Sftp(e) => e.code(),
            Self::Tunnel(e) => e.code(),
            Self::Ai(e) => e.code(),
            Self::Sync(e) => e.code(),
            Self::Io(e) => e.code(),
            Self::Validation(e) => e.code(),
            Self::Ftp(e) => e.code(),
            Self::Pro(e) => e.code(),
            Self::NotImplemented(_) => "CAT-CORE-501",
        }
    }

    /// Upper-case domain segment of `code()`, e.g. `"VAULT"`.
    pub fn domain(&self) -> &'static str {
        match self {
            Self::Vault(_) => "VAULT",
            Self::Db(_) => "DB",
            Self::Ssh(_) => "SSH",
            Self::Sftp(_) => "SFTP",
            Self::Tunnel(_) => "TUNNEL",
            Self::Ai(_) => "AI",
            Self::Sync(_) => "SYNC",
            Self::Io(_) => "IO",
            Self::Validation(_) => "VALIDATION",
            Self::Ftp(_) => "FTP",
            Self::Pro(_) => "PRO",
            Self::NotImplemented(_) => "CORE",
        }
    }

    /// Message safe to render in the UI: free of secrets and internal jargon by
    /// construction, since every leaf variant's `Display` only ever carries a plain
    /// description string, never a raw secret value (REQ-04 §Redaksi).
    pub fn user_message(&self) -> String {
        self.to_string()
    }

    /// One representative instance per currently-defined leaf variant, for tests and for
    /// generating `docs/error-codes.md`. Exhaustive matches in `code()`/`domain()` above
    /// mean adding a new top-level variant without extending this list fails to compile
    /// long before anyone forgets to update the registry.
    pub fn all_known_for_registry() -> Vec<Self> {
        const PLACEHOLDER: &str = "unclassified error in this domain";
        vec![
            Self::Vault(VaultError::Generic(PLACEHOLDER.into())),
            Self::Db(DbError::Generic(PLACEHOLDER.into())),
            Self::Ssh(SshError::Generic(PLACEHOLDER.into())),
            Self::Sftp(SftpError::Generic(PLACEHOLDER.into())),
            Self::Tunnel(TunnelError::Generic(PLACEHOLDER.into())),
            Self::Ai(AiError::Generic(PLACEHOLDER.into())),
            Self::Sync(SyncError::Generic(PLACEHOLDER.into())),
            Self::Io(IoError::Generic(PLACEHOLDER.into())),
            Self::Validation(ValidationError::Generic(PLACEHOLDER.into())),
            Self::Ftp(FtpError::Generic(PLACEHOLDER.into())),
            Self::Pro(ProError::Generic(PLACEHOLDER.into())),
            Self::NotImplemented(PLACEHOLDER.into()),
        ]
    }

    /// Renders the same table checked in at `docs/error-codes.md`. Kept in the library so
    /// both `caframeworkctl errors list` and the doc-freshness test call one source of truth.
    pub fn generate_registry_markdown() -> String {
        let mut out = String::from(
            "# CAFramework v2 — Error Code Registry\n\n\
             Generated from `crates/caf-core/src/error.rs`. Do not edit by hand — \
             regenerate with `UPDATE_GOLDEN=1 cargo test -p caf-core --test error_codes_unique`.\n\n\
             | Code | Domain | Message |\n|---|---|---|\n",
        );
        for err in Self::all_known_for_registry() {
            out.push_str(&format!(
                "| `{}` | {} | {} |\n",
                err.code(),
                err.domain(),
                err.user_message()
            ));
        }
        out
    }
}

impl Serialize for CatermError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("CatermError", 3)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.user_message())?;
        s.serialize_field("domain", self.domain())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_known_code_is_unique() {
        let codes: Vec<&'static str> = CatermError::all_known_for_registry()
            .iter()
            .map(CatermError::code)
            .collect();
        let unique: HashSet<&'static str> = codes.iter().copied().collect();
        assert_eq!(
            codes.len(),
            unique.len(),
            "kode error terduplikasi: {codes:?}"
        );
    }

    #[test]
    fn serialize_shape_has_code_message_domain() {
        let err = CatermError::Validation(ValidationError::Generic("input tidak valid".into()));
        let value = serde_json::to_value(&err).expect("serialize gagal");
        assert_eq!(value["code"], "CAT-VALIDATION-000");
        assert_eq!(value["domain"], "VALIDATION");
        assert!(value["message"].as_str().is_some());
    }

    /// DoD-1: seluruh fungsi publik `caf-core` yang bisa gagal mengembalikan
    /// `Result<T, CatermError>`. Scan tekstual atas `src/**.rs`, mengecualikan modul
    /// `#[cfg(test)]` dan `src/error.rs` sendiri — modul yang MENDEFINISIKAN `CatermError`
    /// tidak bisa mensyaratkan method inherent-nya sendiri (`code()`, `domain()`, ...)
    /// mengembalikan `Result<_, CatermError>`, itu sirkular dan getter murni tidak pernah
    /// gagal secara definisi. Begitu modul lain (vault/store/ssh/...) menambah fungsi publik,
    /// test ini yang menegakkan supaya fungsi fallible-nya lewat `CatermError`, bukan
    /// `unwrap`/`String` ad-hoc/panic.
    #[test]
    fn every_public_fn_in_core_returns_caframework_error_result() {
        let src_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut violations: Vec<String> = Vec::new();
        scan_pub_fns(&src_dir, &mut violations);
        for v in &violations {
            println!("VIOLATION: {}", v);
        }
        assert!(
            violations.is_empty(),
            "PELANGGARAN REQ-04: fungsi publik caf-core tidak mengembalikan Result<_, CatermError>: {violations:?}"
        );
    }

    fn scan_pub_fns(dir: &std::path::Path, violations: &mut Vec<String>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().and_then(|n| n.to_str()) == Some("error.rs") {
                continue;
            }
            if path.is_dir() {
                scan_pub_fns(&path, violations);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(contents) = std::fs::read_to_string(&path) else {
                continue;
            };
            check_file(&path, &contents, violations);
        }
    }

    fn check_file(path: &std::path::Path, contents: &str, violations: &mut Vec<String>) {
        let mut in_test_mod = false;
        let mut test_mod_depth: i32 = -1;
        let mut depth: i32 = 0;
        let lines: Vec<&str> = contents.lines().collect();
        let mut i = 0usize;
        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim_start();

            for ch in line.chars() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if in_test_mod && depth < test_mod_depth {
                        in_test_mod = false;
                    }
                }
            }

            if trimmed.starts_with("#[cfg(test)]")
                && !in_test_mod
                && let Some(next) = lines.get(i + 1)
                && next.trim_start().starts_with("mod ")
            {
                // The next `mod ... {` line enters the excluded test module.
                in_test_mod = true;
                test_mod_depth = depth + 1;
            }

            if !in_test_mod && trimmed.starts_with("pub fn ") {
                // Reconstruct the full signature (may span multiple lines) up to `{`.
                let mut sig = String::new();
                let mut j = i;
                loop {
                    sig.push_str(lines[j]);
                    sig.push(' ');
                    if lines[j].contains('{') || lines[j].trim_end().ends_with(';') {
                        break;
                    }
                    j += 1;
                    if j >= lines.len() {
                        break;
                    }
                }
                let sig_head = sig.split('{').next().unwrap_or(&sig);
                let is_result_of_caframework_error =
                    sig_head.contains("Result<") && sig_head.contains("CatermError");
                let is_marked_infallible = preceded_by_infallible_marker(&lines, i);
                if !is_result_of_caframework_error && !is_marked_infallible {
                    violations.push(format!("{}:{}: {}", path.display(), i + 1, sig_head.trim()));
                }
            }

            i += 1;
        }
    }

    /// A `pub fn` is exempt from the `Result<_, CatermError>` requirement only when the
    /// doc comment immediately above it contains the literal marker `# Infallible` plus a
    /// one-line reason. This keeps every exemption grep-able and deliberate — a function
    /// simply lacking a failure mode today is not the same as one that can never grow one.
    fn preceded_by_infallible_marker(lines: &[&str], pub_fn_line_idx: usize) -> bool {
        let mut j = pub_fn_line_idx;
        while j > 0 {
            j -= 1;
            let t = lines[j].trim_start();
            if t.starts_with("///") {
                if t.contains("# Infallible") {
                    return true;
                }
                continue;
            }
            if t.starts_with("#[") {
                continue;
            }
            break;
        }
        false
    }
}
