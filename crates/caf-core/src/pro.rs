//! Generic Pro Licensing & Account client.

use crate::error::CatermError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProStatus {
    pub is_pro: bool,
    pub plan: String,
    pub email: Option<String>,
    pub expires_at: Option<String>,
}

pub fn get_pro_status() -> Result<ProStatus, CatermError> {
    Ok(ProStatus {
        is_pro: false,
        plan: "free".to_string(),
        email: None,
        expires_at: None,
    })
}
