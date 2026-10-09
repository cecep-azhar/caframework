//! 6-Stage System Prompt and Context Injection Pipeline.

use super::db::{get_default_persona, get_persona, get_skills, search_relevant_habits};
use super::models::{SystemPersona, TerminalContext};
use super::scrubber::PromptScrubber;
use super::skills::CustomSkill;
use crate::error::CafError;
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct AssembledPromptContext {
    pub system_prompt: String,
    pub user_prompt: String,
    pub injected_habits_count: usize,
    pub sanitized_terminal_lines: usize,
    pub active_skill: Option<CustomSkill>,
}

pub struct ContextBuilder<'a> {
    conn: &'a Connection,
    scrubber: PromptScrubber,
}

impl<'a> ContextBuilder<'a> {
    /// # Infallible: wraps reference to database connection
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            scrubber: PromptScrubber::new(),
        }
    }

    /// Assemble full context following the 6-stage lifecycle.
    pub fn build(
        &self,
        user_query: &str,
        persona_id: Option<&str>,
        terminal_ctx: Option<&TerminalContext>,
        max_tokens_budget: u32,
    ) -> Result<AssembledPromptContext, CafError> {
        self.build_with_skill(
            user_query,
            persona_id,
            None,
            terminal_ctx,
            max_tokens_budget,
        )
    }

    /// Assemble full context with optional explicit or trigger-matched skill.
    pub fn build_with_skill(
        &self,
        user_query: &str,
        persona_id: Option<&str>,
        explicit_skill_name: Option<&str>,
        terminal_ctx: Option<&TerminalContext>,
        max_tokens_budget: u32,
    ) -> Result<AssembledPromptContext, CafError> {
        let mut system_sections: Vec<String> = Vec::new();

        // Check for matching skill (explicit or via trigger in query)
        let skills = get_skills(self.conn)?;
        let active_skill: Option<CustomSkill> = if let Some(target) = explicit_skill_name {
            skills.into_iter().find(|s| {
                s.is_enabled
                    && (s.name.eq_ignore_ascii_case(target)
                        || s.name.eq_ignore_ascii_case(target.trim_start_matches('@')))
            })
        } else {
            skills
                .into_iter()
                .find(|s| s.is_enabled && s.matches_trigger(user_query))
        };

        // STAGE 1: Persona Base System Prompt
        let persona: Option<SystemPersona> = if let Some(pid) = persona_id {
            get_persona(self.conn, pid)?
        } else {
            get_default_persona(self.conn)?
        };

        if let Some(p) = &persona {
            let mut sec = format!("[SYSTEM CONTEXT: PERSONA]\n{}\n", p.system_prompt.trim());
            if !p.custom_rules.is_empty() {
                sec.push_str("Rules:\n");
                for rule in &p.custom_rules {
                    sec.push_str(&format!("- {}\n", rule.trim()));
                }
            }
            system_sections.push(sec);
        } else {
            system_sections.push(
                "[SYSTEM CONTEXT: PERSONA]\nYou are CAFramework Master AI Assistant, an elite Linux/DevOps Systems Architect.\nBe concise, direct, and terminal-first.\nAlways provide copy-pasteable bash/zsh commands.\n".to_string()
            );
        }

        // STAGE 2: Active Skill SOP Injection
        if let Some(skill) = &active_skill {
            let mut skill_sec = format!(
                "[ACTIVE CUSTOM SKILL SOP: @{}]\nTitle: {}\nCategory: {}\n{}\n",
                skill.name,
                skill.title,
                skill.category,
                skill.system_instructions.trim()
            );
            if !skill.allowed_tools.is_empty() {
                skill_sec.push_str(&format!(
                    "Allowed Tools: {}\n",
                    skill.allowed_tools.join(", ")
                ));
            }
            system_sections.push(skill_sec);
        }

        // STAGE 3: Environment Constraints & Host Metadata
        let mut env_lines = Vec::new();
        if let Some(p) = &persona {
            if let Some(c) = &p.environment_constraints {
                if !c.trim().is_empty() {
                    env_lines.push(format!("Platform: {}", c.trim()));
                }
            }
        }
        if let Some(t) = terminal_ctx {
            if let Some(h) = &t.host_label {
                env_lines.push(format!("Host: {h}"));
            }
            if let Some(u) = &t.user {
                env_lines.push(format!("User: {u}"));
            }
            if let Some(sh) = &t.shell {
                env_lines.push(format!("Shell: {sh}"));
            }
            if let Some(cwd) = &t.cwd {
                env_lines.push(format!("Current Directory: {cwd}"));
            }
            if let Some(os) = &t.os_info {
                env_lines.push(format!("OS Info: {os}"));
            }
        }
        if !env_lines.is_empty() {
            system_sections.push(format!(
                "[ENVIRONMENT CONSTRAINTS]\n{}\n",
                env_lines.join("\n")
            ));
        }

        // STAGE 4: Semantic Habit Memory Retrieval (FTS5 + Top-K)
        let relevant_habits = search_relevant_habits(self.conn, user_query, 5)?;
        let habit_count = relevant_habits.len();
        if !relevant_habits.is_empty() {
            let mut habit_sec = String::from("[LEARNED USER HABITS & PREFERENCES]\n");
            for h in &relevant_habits {
                habit_sec.push_str(&format!("- {}\n", h.fact_content.trim()));
            }
            system_sections.push(habit_sec);
        }

        // STAGE 5: Terminal Context Sanitization & Budget Clamping
        let mut sanitized_lines_count = 0;
        let mut user_prompt_body = self.scrubber.scrub(user_query);

        if let Some(t) = terminal_ctx {
            if !t.tail_lines.is_empty() {
                // Tail at most 40 lines
                let line_count = t.tail_lines.len();
                let start_idx = line_count.saturating_sub(40);
                let tail: Vec<String> = t.tail_lines.iter().skip(start_idx).cloned().collect();

                let mut scrubbed = self.scrubber.scrub_lines(&tail);
                sanitized_lines_count = scrubbed.len();

                // Approximate budget clamping: 1 token ~ 4 chars. Cap terminal lines to 60% of budget.
                let char_budget = ((max_tokens_budget as usize) * 4 * 60) / 100;
                let mut total_chars = scrubbed.iter().map(|l| l.len()).sum::<usize>();
                while total_chars > char_budget && scrubbed.len() > 5 {
                    if !scrubbed.is_empty() {
                        scrubbed.remove(0);
                    }
                    total_chars = scrubbed.iter().map(|l| l.len()).sum::<usize>();
                }

                let mut term_sec = String::from("\n[RELEVANT TERMINAL OUTPUT (SANITIZED)]\n");
                for line in scrubbed {
                    term_sec.push_str(&format!("{line}\n"));
                }
                if let Some(code) = t.exit_code {
                    term_sec.push_str(&format!("Exit Code: {code}\n"));
                }
                user_prompt_body.push_str(&term_sec);
            }
        }

        // STAGE 6: Assemble final system prompt and user prompt
        let final_system_prompt = system_sections.join("\n");

        Ok(AssembledPromptContext {
            system_prompt: final_system_prompt,
            user_prompt: user_prompt_body,
            injected_habits_count: habit_count,
            sanitized_terminal_lines: sanitized_lines_count,
            active_skill,
        })
    }
}
