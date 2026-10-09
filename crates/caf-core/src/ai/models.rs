//! Data models for Master AI Routing, Custom Context & Persistent Habit Memory.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Chat,
    ErrorDiagnostic,
    PromptStudio,
    CommandAutocomplete,
    SecurityReview,
}

impl TaskType {
    /// # Infallible: static string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::ErrorDiagnostic => "error_diagnostic",
            Self::PromptStudio => "prompt_studio",
            Self::CommandAutocomplete => "command_autocomplete",
            Self::SecurityReview => "security_review",
        }
    }

    /// # Infallible: static string parser
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "chat" => Some(Self::Chat),
            "error_diagnostic" => Some(Self::ErrorDiagnostic),
            "prompt_studio" => Some(Self::PromptStudio),
            "command_autocomplete" => Some(Self::CommandAutocomplete),
            "security_review" => Some(Self::SecurityReview),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    CatermHosted,
    Anthropic,
    OpenaiCompatible,
    Ollama,
}

impl ProviderType {
    /// # Infallible: static string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CatermHosted => "caterm_hosted",
            Self::Anthropic => "anthropic",
            Self::OpenaiCompatible => "openai_compatible",
            Self::Ollama => "ollama",
        }
    }

    /// # Infallible: static string parser
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "caterm_hosted" => Some(Self::CatermHosted),
            "anthropic" => Some(Self::Anthropic),
            "openai_compatible" => Some(Self::OpenaiCompatible),
            "ollama" => Some(Self::Ollama),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub default_model: String,
    pub is_active: bool,
    pub custom_headers: Option<HashMap<String, String>>,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderSummary {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub base_url: String,
    pub default_model: String,
    pub is_active: bool,
    pub has_api_key: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRouteRule {
    pub task_type: TaskType,
    pub primary_provider_id: String,
    pub primary_model: String,
    pub fallback_provider_id: Option<String>,
    pub fallback_model: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub system_persona_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitFact {
    pub id: String,
    pub category: String,
    pub key_tag: String,
    pub fact_content: String,
    pub source_context: Option<String>,
    pub confidence_score: f32,
    pub occurrence_count: u32,
    pub is_pinned: bool,
    pub is_active: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub last_accessed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPersona {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub system_prompt: String,
    pub custom_rules: Vec<String>,
    pub environment_constraints: Option<String>,
    pub is_global_default: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalContext {
    #[serde(default)]
    pub tail_lines: Vec<String>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub host_label: Option<String>,
    #[serde(default)]
    pub active_host_id: Option<String>,
    #[serde(default)]
    pub shell: Option<String>,
    #[serde(default)]
    pub os_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResult {
    pub content: String,
    pub provider_id: String,
    pub provider_type: ProviderType,
    pub model: String,
    pub fell_back: bool,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
}
