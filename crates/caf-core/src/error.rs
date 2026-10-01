//! Generic error taxonomy for CAFramework (CAF-*)

use serde::Serialize;
use thiserror::Error;

macro_rules! domain_error {
    ($name:ident, $domain:literal, $( $variant:ident = ($num:literal, $msg:literal) ),* $(,)?) => {
        #[derive(Debug, Error, Clone, PartialEq, Eq)]
        pub enum $name {
            $(
                #[error($msg)]
                $variant,
            )*
            #[error("{0}")]
            Generic(String),
        }

        impl $name {
            pub fn code(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => concat!("CAF-", $domain, "-", $num),
                    )*
                    Self::Generic(_) => concat!("CAF-", $domain, "-000"),
                }
            }

            pub fn all_variants() -> Vec<Self> {
                vec![
                    $(
                        Self::$variant,
                    )*
                    Self::Generic("sample error".into()),
                ]
            }
        }
    };
}

domain_error!(
    VaultError,
    "VAULT",
    Locked = ("001", "vault is locked"),
    AlreadyUnlocked = ("002", "vault is already unlocked"),
    InvalidPassword = ("003", "invalid master password"),
    LockoutActive = (
        "004",
        "account is temporarily locked out due to too many failed attempts"
    ),
    CorruptedHeader = ("005", "vault database header is corrupted"),
);

domain_error!(
    DbError,
    "DB",
    ConnectionFailed = ("001", "database connection failed"),
    QueryFailed = ("002", "database query execution failed"),
    MigrationFailed = ("003", "database migration failed"),
    RecordNotFound = ("004", "record not found"),
);

domain_error!(
    AiError,
    "AI",
    ProviderUnavailable = ("001", "AI provider service is unavailable"),
    ApiKeyMissing = ("002", "AI provider API key is not configured"),
    GuardrailTriggered = ("003", "input or output violated system guardrails"),
);

domain_error!(
    IoError,
    "IO",
    FileNotFound = ("001", "file not found"),
    PermissionDenied = ("002", "permission denied"),
);

domain_error!(
    ValidationError,
    "VALIDATION",
    EmptyField = ("001", "field cannot be empty"),
    InvalidFormat = ("002", "invalid field format"),
);

domain_error!(
    ProError,
    "PRO",
    FeatureLocked = ("001", "feature requires active Pro license"),
);

domain_error!(
    AuthError,
    "AUTH",
    Unauthorized = ("001", "unauthorized profile access"),
    InvalidPin = ("002", "invalid profile PIN"),
    PermissionDenied = ("003", "insufficient role permissions"),
);

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CafError {
    #[error("vault: {0}")]
    Vault(#[from] VaultError),
    #[error("database: {0}")]
    Db(#[from] DbError),
    #[error("ai: {0}")]
    Ai(#[from] AiError),
    #[error("io: {0}")]
    Io(#[from] IoError),
    #[error("validation: {0}")]
    Validation(#[from] ValidationError),
    #[error("pro: {0}")]
    Pro(#[from] ProError),
    #[error("auth: {0}")]
    Auth(#[from] AuthError),
    #[error("not implemented: {0}")]
    NotImplemented(String),
}

pub type CatermError = CafError;

impl CafError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Vault(e) => e.code(),
            Self::Db(e) => e.code(),
            Self::Ai(e) => e.code(),
            Self::Io(e) => e.code(),
            Self::Validation(e) => e.code(),
            Self::Pro(e) => e.code(),
            Self::Auth(e) => e.code(),
            Self::NotImplemented(_) => "CAF-CORE-501",
        }
    }

    pub fn domain(&self) -> &'static str {
        match self {
            Self::Vault(_) => "VAULT",
            Self::Db(_) => "DB",
            Self::Ai(_) => "AI",
            Self::Io(_) => "IO",
            Self::Validation(_) => "VALIDATION",
            Self::Pro(_) => "PRO",
            Self::Auth(_) => "AUTH",
            Self::NotImplemented(_) => "CORE",
        }
    }

    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Io(IoError::Generic(msg.into()))
    }

    pub fn all_known_for_registry() -> Vec<Self> {
        let mut list = Vec::new();
        for v in VaultError::all_variants() {
            list.push(Self::Vault(v));
        }
        for v in DbError::all_variants() {
            list.push(Self::Db(v));
        }
        for v in AiError::all_variants() {
            list.push(Self::Ai(v));
        }
        for v in IoError::all_variants() {
            list.push(Self::Io(v));
        }
        for v in ValidationError::all_variants() {
            list.push(Self::Validation(v));
        }
        for v in ProError::all_variants() {
            list.push(Self::Pro(v));
        }
        for v in AuthError::all_variants() {
            list.push(Self::Auth(v));
        }
        list.push(Self::NotImplemented("example feature".into()));
        list
    }

    pub fn generate_registry_markdown() -> String {
        let mut md = String::from(
            "# CAFramework Error Code Registry\n\n| Code | Domain | Description |\n| --- | --- | --- |\n",
        );
        let mut items = Self::all_known_for_registry();
        items.sort_by_key(|e| e.code());
        for item in items {
            md.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                item.code(),
                item.domain(),
                item.to_string().replace('|', "\\|")
            ));
        }
        md
    }
}

#[derive(Serialize)]
pub struct ErrorEnvelope {
    pub code: &'static str,
    pub message: String,
    pub domain: &'static str,
}

impl Serialize for CafError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ErrorEnvelope {
            code: self.code(),
            message: self.to_string(),
            domain: self.domain(),
        }
        .serialize(serializer)
    }
}
