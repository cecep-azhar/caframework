//! Custom Skills Engine (SKILL.md parser, registry, and execution SOP injection).
//! Supports YAML frontmatter parsing and standard @skill triggers.

use crate::error::{AiError, CafError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomSkill {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub triggers: Vec<String>,
    pub preferred_model_id: Option<String>,
    pub system_instructions: String,
    pub allowed_tools: Vec<String>,
    pub is_builtin: bool,
    pub is_enabled: bool,
    #[serde(default)]
    pub created_at: i64,
}

impl CustomSkill {
    /// Parse a SKILL.md file format (YAML frontmatter delimited by `---` + Markdown body).
    pub fn from_skill_markdown(content: &str, id: &str) -> Result<Self, CafError> {
        let trimmed = content.trim();
        if !trimmed.starts_with("---") {
            return Err(CafError::Ai(AiError::Generic(
                "SKILL.md must start with YAML frontmatter delimiter '---'".to_string(),
            )));
        }

        let rest = &trimmed[3..];
        let Some(end_idx) = rest.find("\n---") else {
            return Err(CafError::Ai(AiError::Generic(
                "SKILL.md missing closing frontmatter delimiter '---'".to_string(),
            )));
        };

        let frontmatter = &rest[..end_idx];
        let body = rest[end_idx + 4..].trim().to_string();

        let mut name = String::new();
        let mut title = String::new();
        let mut description = String::new();
        let mut category = "general".to_string();
        let mut triggers = Vec::new();
        let mut preferred_model = None;
        let mut tools = Vec::new();

        for line in frontmatter.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                let k = k.trim();
                let v = v.trim();
                match k {
                    "name" => name = v.trim_matches('"').trim_matches('\'').to_string(),
                    "title" => title = v.trim_matches('"').trim_matches('\'').to_string(),
                    "description" => {
                        description = v.trim_matches('"').trim_matches('\'').to_string()
                    }
                    "category" => category = v.trim_matches('"').trim_matches('\'').to_string(),
                    "preferred_model" | "preferred_model_id" => {
                        let m = v.trim_matches('"').trim_matches('\'');
                        if !m.is_empty() {
                            preferred_model = Some(m.to_string());
                        }
                    }
                    "tools" => {
                        tools = parse_simple_list(v);
                    }
                    "triggers" => {
                        triggers = parse_simple_list(v);
                    }
                    _ => {}
                }
            }
        }

        if name.is_empty() {
            return Err(CafError::Ai(AiError::Generic(
                "SKILL.md frontmatter must contain 'name'".to_string(),
            )));
        }

        if title.is_empty() {
            title = name.clone();
        }

        if triggers.is_empty() {
            triggers.push(format!("@{name}"));
        }

        Ok(Self {
            id: id.to_string(),
            name,
            title,
            description,
            category,
            triggers,
            preferred_model_id: preferred_model,
            system_instructions: body,
            allowed_tools: tools,
            is_builtin: false,
            is_enabled: true,
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    /// # Infallible: pure serialization of memory struct to markdown
    /// Render skill to SKILL.md format
    pub fn to_skill_markdown(&self) -> String {
        let triggers_str =
            serde_json::to_string(&self.triggers).unwrap_or_else(|_| "[]".to_string());
        let tools_str =
            serde_json::to_string(&self.allowed_tools).unwrap_or_else(|_| "[]".to_string());
        let pref_model = self.preferred_model_id.as_deref().unwrap_or("");

        format!(
            "---\nname: {}\ntitle: {}\ndescription: {}\ncategory: {}\npreferred_model: {}\ntriggers: {}\ntools: {}\n---\n\n{}",
            self.name,
            self.title,
            self.description,
            self.category,
            pref_model,
            triggers_str,
            tools_str,
            self.system_instructions.trim()
        )
    }

    /// # Infallible: pure string substring matching against trigger list
    /// Check if user prompt matches any triggers in this skill
    pub fn matches_trigger(&self, prompt: &str) -> bool {
        let lower = prompt.to_lowercase();
        self.triggers.iter().any(|t| {
            let t_low = t.to_lowercase();
            let t_clean = t_low.trim_start_matches('@');
            lower.contains(&t_low) || lower.contains(t_clean)
        })
    }
}

fn parse_simple_list(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    if raw.starts_with('[') && raw.ends_with(']') {
        if let Ok(vec) = serde_json::from_str::<Vec<String>>(raw) {
            return vec;
        }
        let inner = &raw[1..raw.len() - 1];
        return inner
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    raw.split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// # Infallible: returns static built-in custom skills definitions
/// Built-in Custom Skills definition
pub fn builtin_skills() -> Vec<CustomSkill> {
    vec![
        CustomSkill {
            id: "builtin-k8s-triage".to_string(),
            name: "k8s-triage".to_string(),
            title: "Kubernetes Pod & CrashLoopBackOff Triage".to_string(),
            description: "Automated root-cause diagnostics and fix commands for failing Kubernetes pods".to_string(),
            category: "devops".to_string(),
            triggers: vec![
                "@k8s-triage".to_string(),
                "analisa pod crash".to_string(),
                "pod crash".to_string(),
                "k8s crash".to_string(),
                "debug pod".to_string(),
                "debug k8s".to_string(),
                "crashloopbackoff".to_string(),
            ],
            preferred_model_id: Some("claude-3-5-haiku".to_string()),
            system_instructions: r#"# Identity & Objective
You are a Senior Kubernetes Site Reliability Engineer (SRE).
Your task is to analyze Kubernetes pods experiencing CrashLoopBackOff, OOMKilled, ImagePullBackOff, or eviction.

# Workflow:
1. Examine terminal logs, exit codes, and event messages provided in the prompt.
2. Determine root cause: (e.g. exit 137 OOM, exit 1 missing config, DNS lookup failure, Readiness/Liveness probe timeout).
3. If logs are insufficient, specify exact `kubectl describe pod <name> -n <ns>` or `kubectl logs <name> -n <ns> --previous`.
4. Provide immediate remediation steps with safe, ready-to-copy bash commands."#.to_string(),
            allowed_tools: vec!["terminal".to_string(), "file_reader".to_string()],
            is_builtin: true,
            is_enabled: true,
            created_at: 1775600000,
        },
        CustomSkill {
            id: "builtin-sre-ops".to_string(),
            name: "sre-ops".to_string(),
            title: "Linux & System SRE Incident Responder".to_string(),
            description: "High-priority Linux incident triage for CPU spikes, disk full, OOM, and systemd failures".to_string(),
            category: "sre".to_string(),
            triggers: vec![
                "@sre-ops".to_string(),
                "sre incident".to_string(),
                "server down".to_string(),
                "disk full".to_string(),
                "high load".to_string(),
            ],
            preferred_model_id: Some("claude-3-7-sonnet".to_string()),
            system_instructions: r#"# Identity & Objective
You are a Staff Linux SRE Incident Commander.
Your objective is to quickly triage critical system outages and performance degradation.

# Protocols:
1. Check resource pressure: Memory (free -h), Disk (df -h), Load (uptime), IO (iostat/vmstat).
2. Check failing systemd units (`systemctl --failed`).
3. Inspect system journal (`journalctl -xeu <service> -n 50 --no-pager`).
4. Recommend non-destructive investigative commands first, followed by clear rollback-safe mitigations."#.to_string(),
            allowed_tools: vec!["terminal".to_string()],
            is_builtin: true,
            is_enabled: true,
            created_at: 1775600000,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_markdown_parse_and_export() {
        let raw = r#"---
name: test-skill
title: Test Custom Skill
description: A skill for testing
category: test
preferred_model: claude-3-5-haiku
triggers: ["@test-skill", "do test"]
tools: ["terminal"]
---

# Instructions
Follow these steps carefully:
1. Run test
2. Report status
"#;

        let skill = CustomSkill::from_skill_markdown(raw, "skill-1").expect("parse failed");
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.title, "Test Custom Skill");
        assert_eq!(skill.category, "test");
        assert_eq!(
            skill.preferred_model_id.as_deref(),
            Some("claude-3-5-haiku")
        );
        assert!(skill.triggers.contains(&"@test-skill".to_string()));
        assert!(skill.allowed_tools.contains(&"terminal".to_string()));
        assert!(
            skill
                .system_instructions
                .contains("Follow these steps carefully:")
        );

        let exported = skill.to_skill_markdown();
        assert!(exported.contains("name: test-skill"));
        assert!(exported.contains("# Instructions"));
    }

    #[test]
    fn test_trigger_match() {
        let skills = builtin_skills();
        let k8s = skills.iter().find(|s| s.name == "k8s-triage").unwrap();
        assert!(k8s.matches_trigger("tolong analisa pod crash di cluster staging"));
        assert!(k8s.matches_trigger("halo @k8s-triage tolong cek"));
        assert!(!k8s.matches_trigger("halo selamat pagi"));
    }
}
