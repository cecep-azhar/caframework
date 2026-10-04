//! Visibility filter, ownership evaluation and scoped queries for CAFramework v2.
//! Implements multi-profile data isolation, private summary views, and audited super-admin bypass.

use crate::error::{CatermError, ValidationError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Shared,
    PrivateSummary,
    Private,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Shared => "shared",
            Visibility::PrivateSummary => "private_summary",
            Visibility::Private => "private",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, CatermError> {
        match s {
            "shared" => Ok(Visibility::Shared),
            "private_summary" => Ok(Visibility::PrivateSummary),
            "private" => Ok(Visibility::Private),
            _ => Err(CatermError::Validation(ValidationError::InvalidFormat)),
        }
    }
}

/// Computes the SQL filter for queries returning entities.
///
/// If `is_super_role` is true, visibility isolation is relaxed (super admin bypass)
/// while deleted_at IS NULL is maintained.
pub fn sql_scope(is_super_role: bool) -> &'static str {
    if is_super_role {
        "deleted_at IS NULL"
    } else {
        "deleted_at IS NULL AND (owner_profile_id = ?1 OR visibility = 'shared')"
    }
}

/// Determines whether a profile can modify or delete an entity row.
pub fn can_modify(owner_profile_id: &str, current_profile_id: &str, has_permission: bool) -> bool {
    owner_profile_id == current_profile_id || has_permission
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_parsing() {
        assert_eq!(Visibility::from_str("shared").unwrap(), Visibility::Shared);
        assert_eq!(
            Visibility::from_str("private_summary").unwrap(),
            Visibility::PrivateSummary
        );
        assert_eq!(
            Visibility::from_str("private").unwrap(),
            Visibility::Private
        );
        assert!(Visibility::from_str("invalid").is_err());
    }

    #[test]
    fn test_sql_scope() {
        assert_eq!(sql_scope(true), "deleted_at IS NULL");
        assert_eq!(
            sql_scope(false),
            "deleted_at IS NULL AND (owner_profile_id = ?1 OR visibility = 'shared')"
        );
    }

    #[test]
    fn test_can_modify() {
        assert!(can_modify("prof_1", "prof_1", false));
        assert!(!can_modify("prof_1", "prof_2", false));
        assert!(can_modify("prof_1", "prof_2", true));
    }
}
