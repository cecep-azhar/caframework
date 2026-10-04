//! Visibility rules, data isolation, and super-role audit scopes.

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
            Self::Shared => "shared",
            Self::PrivateSummary => "private_summary",
            Self::Private => "private",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "shared" => Some(Self::Shared),
            "private_summary" => Some(Self::PrivateSummary),
            "private" => Some(Self::Private),
            _ => None,
        }
    }
}

pub struct VisibilityScope {
    pub sql_clause: String,
    pub is_super: bool,
}

pub fn sql_scope(is_super: bool) -> String {
    if is_super {
        "deleted_at IS NULL".to_string()
    } else {
        "deleted_at IS NULL AND (owner_profile_id = ?1 OR visibility = 'shared')".to_string()
    }
}

/// Generates SQL WHERE clause filter for scoped entity queries.
pub fn scope(caller_profile_id: &str, is_super: bool) -> VisibilityScope {
    if is_super {
        VisibilityScope {
            sql_clause: "deleted_at IS NULL".to_string(),
            is_super: true,
        }
    } else {
        VisibilityScope {
            sql_clause: format!(
                "deleted_at IS NULL AND (owner_profile_id = '{}' OR visibility = 'shared')",
                caller_profile_id.replace('\'', "''")
            ),
            is_super: false,
        }
    }
}

pub fn can_modify(owner_profile_id: &str, caller_profile_id: &str, has_manage_perm: bool) -> bool {
    owner_profile_id == caller_profile_id || has_manage_perm
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OwnerSummary {
    pub owner_profile_id: String,
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_scope_regular_user() {
        let sc = scope("user_1", false);
        assert!(!sc.is_super);
        assert!(sc.sql_clause.contains("owner_profile_id = 'user_1'"));
        assert!(sc.sql_clause.contains("visibility = 'shared'"));
    }

    #[test]
    fn test_visibility_scope_super_user() {
        let sc = scope("admin_1", true);
        assert!(sc.is_super);
        assert_eq!(sc.sql_clause, "deleted_at IS NULL");
    }

    #[test]
    fn test_can_modify() {
        assert!(can_modify("user_1", "user_1", false));
        assert!(!can_modify("user_1", "user_2", false));
        assert!(can_modify("user_1", "user_2", true));
    }
}
