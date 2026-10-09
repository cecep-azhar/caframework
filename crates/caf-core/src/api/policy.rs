//! Command policy classification and registry.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Policy {
    /// Works while vault is locked (setup, initial unlock, status, app info).
    Public,
    /// Vault must be unlocked, but no profile session required.
    Unlocked,
    /// Vault must be unlocked and an active profile session must exist.
    Session,
    /// Specific RBAC permission required on the active profile session.
    Requires(&'static str),
}

/// Metadata describing a registered command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPolicy {
    pub name: &'static str,
    pub policy: Policy,
    pub returns_list: bool,
}

pub const COMMAND_POLICIES: &[CommandPolicy] = &[
    // Window management (Public)
    CommandPolicy {
        name: "window_minimize",
        policy: Policy::Public,
        returns_list: false,
    },
    CommandPolicy {
        name: "window_maximize",
        policy: Policy::Public,
        returns_list: false,
    },
    CommandPolicy {
        name: "window_close",
        policy: Policy::Public,
        returns_list: false,
    },
    CommandPolicy {
        name: "window_start_dragging",
        policy: Policy::Public,
        returns_list: false,
    },
    // Vault commands
    CommandPolicy {
        name: "is_vault_initialized",
        policy: Policy::Public,
        returns_list: false,
    },
    CommandPolicy {
        name: "validate_vault_password",
        policy: Policy::Public,
        returns_list: false,
    },
    CommandPolicy {
        name: "lock_vault",
        policy: Policy::Public,
        returns_list: false,
    },
    CommandPolicy {
        name: "change_master_password",
        policy: Policy::Requires("security:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "reset_vault",
        policy: Policy::Requires("security:manage"),
        returns_list: false,
    },
    // Profiles
    CommandPolicy {
        name: "list_profiles",
        policy: Policy::Unlocked,
        returns_list: true,
    },
    CommandPolicy {
        name: "save_profile",
        policy: Policy::Session,
        returns_list: false,
    },
    CommandPolicy {
        name: "verify_pin",
        policy: Policy::Unlocked,
        returns_list: false,
    },
    // Notes
    CommandPolicy {
        name: "list_notes",
        policy: Policy::Session,
        returns_list: true,
    },
    CommandPolicy {
        name: "save_note",
        policy: Policy::Session,
        returns_list: false,
    },
    CommandPolicy {
        name: "delete_note",
        policy: Policy::Session,
        returns_list: false,
    },
    // AI
    CommandPolicy {
        name: "get_ai_settings",
        policy: Policy::Session,
        returns_list: false,
    },
    CommandPolicy {
        name: "save_ai_settings",
        policy: Policy::Requires("settings:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "set_ai_api_key",
        policy: Policy::Requires("ai:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "clear_ai_api_key",
        policy: Policy::Requires("ai:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_preview_context",
        policy: Policy::Session,
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_chat",
        policy: Policy::Session,
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_get_providers",
        policy: Policy::Session,
        returns_list: true,
    },
    CommandPolicy {
        name: "ai_save_provider",
        policy: Policy::Requires("ai:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_delete_provider",
        policy: Policy::Requires("ai:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_get_routing_matrix",
        policy: Policy::Session,
        returns_list: true,
    },
    CommandPolicy {
        name: "ai_save_routing_rule",
        policy: Policy::Requires("ai:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_get_personas",
        policy: Policy::Session,
        returns_list: true,
    },
    CommandPolicy {
        name: "ai_save_persona",
        policy: Policy::Requires("ai:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_delete_persona",
        policy: Policy::Requires("ai:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_get_habits",
        policy: Policy::Session,
        returns_list: true,
    },
    CommandPolicy {
        name: "ai_search_habits",
        policy: Policy::Session,
        returns_list: true,
    },
    CommandPolicy {
        name: "ai_toggle_habit_pin",
        policy: Policy::Session,
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_delete_habit",
        policy: Policy::Session,
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_get_skills",
        policy: Policy::Session,
        returns_list: true,
    },
    CommandPolicy {
        name: "ai_save_skill",
        policy: Policy::Requires("ai:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_delete_skill",
        policy: Policy::Requires("ai:manage"),
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_dispatch_task",
        policy: Policy::Session,
        returns_list: false,
    },
    CommandPolicy {
        name: "ai_dispatch_task_with_skill",
        policy: Policy::Session,
        returns_list: false,
    },
    // Feedback & Crash
    CommandPolicy {
        name: "submit_feedback",
        policy: Policy::Public,
        returns_list: false,
    },
    CommandPolicy {
        name: "get_pending_crash_report",
        policy: Policy::Public,
        returns_list: false,
    },
    CommandPolicy {
        name: "dismiss_crash_report",
        policy: Policy::Public,
        returns_list: false,
    },
    // Backup
    CommandPolicy {
        name: "export_encrypted_backup",
        policy: Policy::Requires("backup:export"),
        returns_list: false,
    },
    CommandPolicy {
        name: "import_encrypted_backup",
        policy: Policy::Requires("backup:import"),
        returns_list: false,
    },
    // Preferences
    CommandPolicy {
        name: "get_performance_prefs",
        policy: Policy::Session,
        returns_list: false,
    },
    CommandPolicy {
        name: "set_performance_prefs",
        policy: Policy::Session,
        returns_list: false,
    },
];

pub fn get_command_policy(command_name: &str) -> Option<&'static CommandPolicy> {
    COMMAND_POLICIES.iter().find(|cp| cp.name == command_name)
}
