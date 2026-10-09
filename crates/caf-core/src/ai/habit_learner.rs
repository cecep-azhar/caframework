//! Heuristic habit learner that automatically observes CLI patterns, flags, and fixes.

use super::db::{get_habits, save_habit};
use super::models::HabitFact;
use crate::error::CafError;
use rusqlite::Connection;

pub struct HabitLearner;

impl HabitLearner {
    /// Inspect an executed command line and optionally its exit code to extract learned preferences.
    pub fn observe_command(
        conn: &Connection,
        command: &str,
        exit_code: Option<i32>,
        host_label: Option<&str>,
    ) -> Result<Option<HabitFact>, CafError> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        // Only learn from successful commands (exit code 0 or unknown)
        if let Some(code) = exit_code {
            if code != 0 {
                return Ok(None);
            }
        }

        let learned = if trimmed.contains("podman-compose")
            || (trimmed.contains("podman") && trimmed.contains("compose"))
        {
            Some((
                "cli_preference",
                "container_runtime",
                "Prefers `podman-compose` / `podman` over legacy Docker.",
            ))
        } else if trimmed.contains("podman ") {
            Some((
                "cli_preference",
                "container_runtime",
                "Uses `podman` as the primary OCI container engine.",
            ))
        } else if trimmed.contains("docker compose") {
            Some((
                "cli_preference",
                "container_runtime",
                "Uses modern Docker Compose v2 plugin (`docker compose`).",
            ))
        } else if trimmed.starts_with("bat ") || trimmed.starts_with("batcat ") {
            Some((
                "cli_preference",
                "file_viewer",
                "Prefers `bat` (syntax-highlighted cat) for viewing files.",
            ))
        } else if trimmed.starts_with("rg ") {
            Some((
                "cli_preference",
                "search_tool",
                "Uses `rg` (ripgrep) for fast recursive code searching.",
            ))
        } else if trimmed.starts_with("fd ") {
            Some((
                "cli_preference",
                "find_tool",
                "Uses `fd` instead of traditional `find`.",
            ))
        } else if trimmed.starts_with("eza ") || trimmed.starts_with("exa ") {
            Some((
                "cli_preference",
                "list_tool",
                "Prefers `eza`/`exa` with modern icon and git flags over `ls`.",
            ))
        } else if trimmed.starts_with("nvim ") {
            Some((
                "cli_preference",
                "default_editor",
                "Uses Neovim (`nvim`) as primary modal editor.",
            ))
        } else if trimmed.starts_with("systemctl ") {
            Some((
                "cli_preference",
                "init_system",
                "Uses `systemctl` for daemon and unit management.",
            ))
        } else if trimmed.starts_with("pnpm ") {
            Some((
                "cli_preference",
                "package_manager",
                "Prefers `pnpm` for fast, hardlinked Node.js package management.",
            ))
        } else if trimmed.starts_with("bun ") {
            Some((
                "cli_preference",
                "javascript_runtime",
                "Uses `bun` for TypeScript/JavaScript execution and bundling.",
            ))
        } else if trimmed.starts_with("uv ") {
            Some((
                "cli_preference",
                "python_tooling",
                "Uses `uv` for fast Python environment and package management.",
            ))
        } else {
            None
        };

        let Some((category, key_tag, fact_content)) = learned else {
            return Ok(None);
        };

        // Check if habit with key_tag already exists
        let habits = get_habits(conn)?;
        if let Some(existing) = habits.iter().find(|h| h.key_tag == key_tag) {
            let mut updated = existing.clone();
            updated.occurrence_count = updated.occurrence_count.saturating_add(1);
            updated.confidence_score = (updated.confidence_score + 0.05).min(1.0);
            save_habit(conn, &updated)?;
            return Ok(Some(updated));
        }

        let new_habit = HabitFact {
            id: uuid::Uuid::new_v4().to_string(),
            category: category.to_string(),
            key_tag: key_tag.to_string(),
            fact_content: fact_content.to_string(),
            source_context: host_label.map(|h| format!("Observed from terminal on {h}")),
            confidence_score: 0.8,
            occurrence_count: 1,
            is_pinned: false,
            is_active: true,
            created_at: 0,
            last_accessed_at: 0,
        };

        save_habit(conn, &new_habit)?;
        Ok(Some(new_habit))
    }
}
