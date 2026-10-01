//! Generic error taxonomy for CAFramework (CAF-*)

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
                    Self::Generic(_) => concat!("CAF-", $domain, "-000"),
                }
            }
        }
    };
}

domain_error!(VaultError, "VAULT");
domain_error!(DbError, "DB");
domain_error!(AiError, "AI");
domain_error!(IoError, "IO");
domain_error!(ValidationError, "VALIDATION");
domain_error!(ProError, "PRO");
domain_error!(AuthError, "AUTH");

#[derive(Debug, Error)]
pub enum CatermError {
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

impl CatermError {
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
}

#[derive(Serialize)]
pub struct ErrorEnvelope {
    pub code: &'static str,
    pub message: String,
    pub domain: &'static str,
}

impl Serialize for CatermError {
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
