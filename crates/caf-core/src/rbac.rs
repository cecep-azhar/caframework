//! RBAC permission matrix definitions and evaluation.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }

    pub fn matches(&self, required: &Permission) -> bool {
        if self.resource == "*" && self.action == "*" {
            return true;
        }
        if self.resource == required.resource
            && (self.action == "*" || self.action == required.action)
        {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct Role {
    pub key: String,
    pub label_en: String,
    pub label_id: String,
    pub is_builtin: bool,
    pub permissions: HashSet<Permission>,
}

impl Role {
    pub fn has_permission(&self, required: &Permission) -> bool {
        self.permissions.iter().any(|p| p.matches(required))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_permissions() {
        let super_admin_perm = Permission::new("*", "*");
        let req = Permission::new("users", "delete");
        assert!(super_admin_perm.matches(&req));

        let user_all_perm = Permission::new("users", "*");
        assert!(user_all_perm.matches(&req));

        let user_view_perm = Permission::new("users", "view");
        assert!(!user_view_perm.matches(&req));
    }
}
