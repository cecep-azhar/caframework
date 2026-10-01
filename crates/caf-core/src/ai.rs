//! Backend AI automation and execution engine for CAFramework v2.
//! Supports AI settings persistence, template and LLM-assisted execution plan generation,
//! and SSH-based automated step execution.

use crate::error::{AiError, CatermError, DbError, SshError, ValidationError};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiPlanStep {
    pub step: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub title: String,
    pub command: String,
    pub description: String,
    #[serde(alias = "isDangerous", alias = "isDanger")]
    pub is_dangerous: bool,
    #[serde(default, alias = "isSudo", skip_serializing_if = "Option::is_none")]
    pub is_sudo: Option<bool>,
    #[serde(default, alias = "stepNumber", skip_serializing_if = "Option::is_none")]
    pub step_number: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_danger: Option<bool>,
}

impl AiPlanStep {
    pub(crate) fn new(
        step: u32,
        title: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
        is_dangerous: bool,
    ) -> Self {
        let cmd = command.into();
        let is_sudo = cmd.contains("sudo");
        Self {
            step,
            id: Some(format!("step-{step}")),
            title: title.into(),
            command: cmd,
            description: description.into(),
            is_dangerous,
            is_sudo: Some(is_sudo),
            step_number: Some(step),
            is_danger: Some(is_dangerous),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiExecutionPlan {
    pub id: String,
    pub goal: String,
    pub summary: String,
    #[serde(default, alias = "hostId")]
    pub host_id: Option<String>,
    #[serde(default, alias = "hostLabel")]
    pub host_label: Option<String>,
    pub requirements: Vec<String>,
    pub steps: Vec<AiPlanStep>,
    #[serde(alias = "createdAt")]
    pub created_at: i64,
    #[serde(
        default,
        alias = "estimatedTime",
        skip_serializing_if = "Option::is_none"
    )]
    pub estimated_time: Option<String>,
    /// Where the plan actually came from: `"llm"` or `"builtin"`.
    ///
    /// The LLM path falls back to the built-in template planner whenever the endpoint cannot be
    /// reached or its answer cannot be parsed. That fallback is useful, but it was invisible:
    /// a broken endpoint still produced a polished plan, so the assistant looked connected when
    /// it was not. The UI reads this to say which one you are looking at.
    #[serde(default = "default_plan_source")]
    pub source: String,
}

fn default_plan_source() -> String {
    "builtin".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiExecutionResult {
    pub step: u32,
    pub command: String,
    pub success: bool,
    #[serde(alias = "exitCode")]
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    #[serde(default, alias = "stepNumber", skip_serializing_if = "Option::is_none")]
    pub step_number: Option<u32>,
    #[serde(default, alias = "durationMs", skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiSettings {
    pub provider: String,
    #[serde(alias = "baseUrl")]
    pub base_url: String,
    #[serde(alias = "apiKey")]
    pub api_key: String,
    pub model: String,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            provider: "custom".to_string(),
            base_url: "http://100.76.150.46:3007/v1".to_string(),
            api_key: String::new(),
            model: "claude-3-5-sonnet".to_string(),
        }
    }
}

pub(crate) fn get_ai_settings_in(conn: &Connection) -> Result<AiSettings, CatermError> {
    let mut stmt = conn
        .prepare("SELECT provider, api_key, base_url, model FROM ai_settings WHERE id = 'default'")
        .map_err(|e| CatermError::Db(DbError::Generic(format!("Failed to prepare query: {e}"))))?;

    let res = stmt.query_row([], |row| {
        Ok(AiSettings {
            provider: row.get(0)?,
            api_key: row.get(1)?,
            base_url: row.get(2)?,
            model: row.get(3)?,
        })
    });

    match res {
        Ok(settings) => Ok(settings),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AiSettings::default()),
        Err(e) => Err(CatermError::Db(DbError::Generic(format!(
            "Failed to load AI settings: {e}"
        )))),
    }
}

pub(crate) fn save_ai_settings_in(
    conn: &Connection,
    settings: AiSettings,
) -> Result<AiSettings, CatermError> {
    conn.execute(
        "INSERT OR REPLACE INTO ai_settings (id, provider, api_key, base_url, model)
         VALUES ('default', ?1, ?2, ?3, ?4);",
        params![
            settings.provider,
            settings.api_key,
            settings.base_url,
            settings.model
        ],
    )
    .map_err(|e| CatermError::Db(DbError::Generic(format!("Failed to save AI settings: {e}"))))?;

    Ok(settings)
}

/// Retrieve AI settings from the encrypted database. Returns defaults if unconfigured.
pub fn get_ai_settings() -> Result<AiSettings, CatermError> {
    let conn = crate::db::open()?;
    get_ai_settings_in(&conn)
}

/// Persist updated AI settings to the encrypted database.
pub fn save_ai_settings(settings: AiSettings) -> Result<AiSettings, CatermError> {
    let conn = crate::db::open()?;
    save_ai_settings_in(&conn, settings)
}

fn resolve_host_label(target_host: Option<&str>) -> Option<String> {
    let host_id = target_host?;
    let conn = crate::db::open().ok()?;
    let mut stmt = conn.prepare("SELECT label FROM hosts WHERE id = ?1").ok()?;
    stmt.query_row(params![host_id], |row| row.get::<_, String>(0))
        .ok()
        .or_else(|| Some(host_id.to_string()))
}

/// POSTs an OpenAI-compatible `/chat/completions` request and returns the assistant's message
/// content.
///
/// This used to shell out to `curl`, passing the API key as `-H "Authorization: Bearer ..."`.
/// Command-line arguments are world-readable on both Windows and Linux, so the key was visible
/// to any local process listing. Sending it from inside our own process closes that, and drops
/// the dependency on `curl.exe` being present.
fn post_chat_completion(
    settings: &AiSettings,
    payload: serde_json::Value,
    timeout: std::time::Duration,
) -> Result<String, CatermError> {
    let base_url = settings.base_url.trim();
    if base_url.is_empty() {
        return Err(CatermError::Ai(AiError::Generic(
            "Base URL AI belum diisi — buka Settings > AI Assistant.".to_string(),
        )));
    }
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build();
    let agent: ureq::Agent = config.into();

    let mut request = agent.post(&url).header("Content-Type", "application/json");
    let api_key = settings.api_key.trim();
    if !api_key.is_empty() {
        request = request.header("Authorization", &format!("Bearer {api_key}"));
    }

    let mut response = request.send_json(&payload).map_err(|e| {
        CatermError::Ai(AiError::Generic(format!("Failed to connect to {url}: {e}")))
    })?;

    let body = response.body_mut().read_to_string().map_err(|e| {
        CatermError::Ai(AiError::Generic(format!("Failed to read AI response: {e}")))
    })?;

    extract_message_content(&body).ok_or_else(|| {
        CatermError::Ai(AiError::Generic(format!(
            "AI response does not match OpenAI chat/completions format: {}",
            body.chars().take(300).collect::<String>()
        )))
    })
}

/// Pulls the assistant's text out of an OpenAI-compatible response.
///
/// Handles both shapes a gateway may answer with: a single JSON object holding
/// `choices[0].message.content`, and a `text/event-stream` of `data:` chunks carrying
/// `choices[0].delta.content` to be concatenated. We request `stream: false`, but a server that
/// streams regardless would otherwise look like a total failure — which is exactly how this
/// surfaced: every request "succeeded" with HTTP 200 and then failed to parse.
fn extract_message_content(body: &str) -> Option<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        return value
            .get("choices")?
            .get(0)?
            .get("message")?
            .get("content")?
            .as_str()
            .map(str::to_string);
    }
    extract_streamed_content(body)
}

/// Reassembles an SSE body. Returns `None` when no chunk carried any text, so a malformed or
/// empty stream is reported as an error rather than as a silent empty answer.
fn extract_streamed_content(body: &str) -> Option<String> {
    let mut out = String::new();

    for line in body.lines() {
        let Some(payload) = line.trim_start().strip_prefix("data:") else {
            continue;
        };
        let payload = payload.trim();
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }
        let Ok(chunk) = serde_json::from_str::<serde_json::Value>(payload) else {
            continue;
        };
        let Some(choice) = chunk.get("choices").and_then(|c| c.get(0)) else {
            continue;
        };
        // `delta.content` while streaming; `message.content` if the server mixes shapes.
        let text = choice
            .get("delta")
            .and_then(|d| d.get("content"))
            .or_else(|| choice.get("message").and_then(|m| m.get("content")))
            .and_then(|c| c.as_str());
        if let Some(text) = text {
            out.push_str(text);
        }
    }

    if out.is_empty() { None } else { Some(out) }
}

/// Strips a ```json ... ``` fence if the model wrapped its answer in one.
fn strip_code_fence(content: &str) -> &str {
    let trimmed = content.trim();
    if let Some(stripped) = trimmed.strip_prefix("```json") {
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else if let Some(stripped) = trimmed.strip_prefix("```") {
        stripped.strip_suffix("```").unwrap_or(stripped).trim()
    } else {
        trimmed
    }
}

const PLAN_SYSTEM_PROMPT: &str = "You are a Linux DevOps and Sysadmin AI assistant. Generate a structured execution plan for the requested goal. Respond ONLY with valid JSON matching this schema:
{
  \"summary\": \"Brief summary of the plan\",
  \"requirements\": [\"Requirement 1\", \"Requirement 2\"],
  \"steps\": [
    {
      \"step\": 1,
      \"title\": \"Step title\",
      \"command\": \"bash command\",
      \"description\": \"Step description\",
      \"is_dangerous\": false
    }
  ]
}";

/// Builds the one-shot plan-generation payload. `model` is omitted for the hosted backend,
/// which assigns the pooled model itself.
fn plan_payload(model: Option<&str>, goal: &str) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "messages": [
            { "role": "system", "content": PLAN_SYSTEM_PROMPT },
            { "role": "user", "content": goal }
        ],
        "temperature": 0.2,
        "stream": false
    });
    if let Some(model) = model
        && let Some(obj) = payload.as_object_mut()
    {
        obj.insert(
            "model".to_string(),
            serde_json::Value::String(model.to_string()),
        );
    }
    payload
}

fn call_llm_if_available(
    settings: &AiSettings,
    goal: &str,
    target_host: Option<&str>,
    host_label: Option<String>,
) -> Option<AiExecutionPlan> {
    let payload = plan_payload(Some(&settings.model), goal);

    // Falls back to the built-in heuristic planner when the endpoint is unreachable, so this
    // deliberately swallows the error rather than surfacing it.
    let content =
        post_chat_completion(settings, payload, std::time::Duration::from_secs(60)).ok()?;
    parse_ai_plan_content(&content, goal, target_host, host_label)
}

/// Same plan generation through GCC's pooled hosted-AI proxy (OmniRoute) instead of a
/// self-configured endpoint. Unlike the BYO path this does not silently fall back to the
/// template planner on failure: the caller opted into hosted AI and spent quota on it, so a
/// quota or network error should reach them, not be hidden behind a generic template.
fn call_llm_hosted(
    goal: &str,
    target_host: Option<&str>,
    host_label: Option<String>,
) -> Result<AiExecutionPlan, CatermError> {
    let payload = plan_payload(None, goal);
    let body = crate::pro::ai_chat_completion(&payload)?;
    let content = extract_message_content(&body.to_string()).ok_or_else(|| {
        CatermError::Ai(AiError::Generic(format!(
            "AI response does not match OpenAI chat/completions format: {}",
            body.to_string().chars().take(300).collect::<String>()
        )))
    })?;
    parse_ai_plan_content(&content, goal, target_host, host_label).ok_or_else(|| {
        CatermError::Ai(AiError::Generic(
            "Model tidak mengembalikan rencana yang bisa dipakai.".to_string(),
        ))
    })
}

/// Reads a `steps: [...]` array into `AiPlanStep`s, skipping entries with no command. Shared by
/// the one-shot planner and the conversational path.
fn parse_plan_steps(parsed: &serde_json::Value) -> Vec<AiPlanStep> {
    let Some(steps_arr) = parsed.get("steps").and_then(|s| s.as_array()) else {
        return Vec::new();
    };

    let mut steps = Vec::new();
    for (idx, item) in steps_arr.iter().enumerate() {
        let step_num = item
            .get("step")
            .and_then(|s| s.as_u64())
            .unwrap_or((idx + 1) as u64) as u32;
        let title = item
            .get("title")
            .and_then(|s| s.as_str())
            .unwrap_or("Execute step")
            .to_string();
        let command = item
            .get("command")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        if command.trim().is_empty() {
            continue;
        }
        let description = item
            .get("description")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();
        let is_dangerous = item
            .get("is_dangerous")
            .and_then(|s| s.as_bool())
            .unwrap_or(false);

        steps.push(AiPlanStep::new(
            step_num,
            title,
            command,
            description,
            is_dangerous,
        ));
    }

    steps
}

/// Parses the assistant's message content (already unwrapped from the HTTP envelope) into a plan.
fn parse_ai_plan_content(
    content: &str,
    goal: &str,
    target_host: Option<&str>,
    host_label: Option<String>,
) -> Option<AiExecutionPlan> {
    let parsed: serde_json::Value = serde_json::from_str(strip_code_fence(content)).ok()?;
    let summary = parsed.get("summary")?.as_str()?.to_string();

    let requirements = parsed
        .get("requirements")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let steps = parse_plan_steps(&parsed);

    if steps.is_empty() {
        return None;
    }

    let now = chrono::Utc::now().timestamp_millis();
    let est = format!("~{} mins", (steps.len() * 2).max(1));

    Some(AiExecutionPlan {
        id: format!("plan-{}", uuid::Uuid::new_v4()),
        goal: goal.to_string(),
        summary,
        host_id: target_host.map(|h| h.to_string()),
        host_label,
        requirements,
        steps,
        // The only place a plan is genuinely the model's work.
        source: "llm".to_string(),
        created_at: now,
        estimated_time: Some(est),
    })
}

fn build_template_plan(
    goal: &str,
    target_host: Option<&str>,
    host_label: Option<String>,
) -> AiExecutionPlan {
    let goal_lower = goal.to_lowercase();
    let now = chrono::Utc::now().timestamp_millis();

    if goal_lower.contains("laravel")
        || goal_lower.contains("php")
        || goal_lower.contains("composer")
        || (goal_lower.contains("ubuntu")
            && (goal_lower.contains("web") || goal_lower.contains("stack")))
    {
        let requirements = vec![
            "PHP 8.3 CLI & FPM".to_string(),
            "Extensions: mbstring, xml, curl, zip, mysql, bcmath, intl, gd, sqlite3".to_string(),
            "Composer v2".to_string(),
            "MySQL server".to_string(),
            "Node.js 20 & NPM".to_string(),
            "Git".to_string(),
        ];

        let steps = vec![
            AiPlanStep::new(
                1,
                "Update system packages",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get update -y && sudo apt-get install -y curl git unzip software-properties-common ca-certificates",
                "Update apt cache and install base prerequisite packages",
                false,
            ),
            AiPlanStep::new(
                2,
                "Add ondrej/php PPA",
                "sudo add-apt-repository -y ppa:ondrej/php && sudo apt-get update -y",
                "Add Ondřej Surý PPA repository for PHP 8.3",
                false,
            ),
            AiPlanStep::new(
                3,
                "Install PHP 8.3 and extensions",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y php8.3 php8.3-cli php8.3-fpm php8.3-mbstring php8.3-xml php8.3-curl php8.3-zip php8.3-mysql php8.3-bcmath php8.3-intl php8.3-gd php8.3-sqlite3",
                "Install PHP 8.3 core, FPM, and required extensions (mbstring, xml, curl, zip, mysql, bcmath, intl, gd, sqlite3)",
                false,
            ),
            AiPlanStep::new(
                4,
                "Install Composer v2",
                "curl -sS https://getcomposer.org/installer | php && sudo mv composer.phar /usr/local/bin/composer && sudo chmod +x /usr/local/bin/composer",
                "Download and install Composer v2 package manager globally",
                false,
            ),
            AiPlanStep::new(
                5,
                "Install MySQL Server",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y mysql-server && sudo systemctl enable --now mysql",
                "Install MySQL server package and start the daemon service",
                false,
            ),
            AiPlanStep::new(
                6,
                "Install Node.js 20 & NPM",
                "curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y nodejs",
                "Set up NodeSource repository and install Node.js 20 LTS and NPM for frontend asset compilation",
                false,
            ),
            AiPlanStep::new(
                7,
                "Verify versions",
                "php -v && composer --version && mysql --version && node -v && npm -v",
                "Verify installed versions of PHP 8.3, Composer, MySQL, and Node.js",
                false,
            ),
        ];

        AiExecutionPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            goal: goal.to_string(),
            summary: "Set up production-ready Laravel environment with PHP 8.3, Composer v2, MySQL, and Node.js 20 on Ubuntu.".to_string(),
            host_id: target_host.map(|h| h.to_string()),
            host_label,
            requirements,
            steps,
            source: default_plan_source(),
        created_at: now,
            estimated_time: Some("~12 mins".to_string()),
        }
    } else if goal_lower.contains("docker") || goal_lower.contains("container") {
        let requirements = vec![
            "Docker Engine (Community)".to_string(),
            "Docker Compose Plugin".to_string(),
            "Containerd runtime".to_string(),
            "Systemd service integration".to_string(),
        ];

        let steps = vec![
            AiPlanStep::new(
                1,
                "Update apt and install prerequisites",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get update -y && sudo apt-get install -y ca-certificates curl gnupg lsb-release",
                "Install system certificates and tools required for Docker GPG keys",
                false,
            ),
            AiPlanStep::new(
                2,
                "Configure Docker GPG key",
                "sudo install -m 0755 -d /etc/apt/keyrings && curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor --yes -o /etc/apt/keyrings/docker.gpg && sudo chmod a+r /etc/apt/keyrings/docker.gpg",
                "Add official Docker repository signing key",
                false,
            ),
            AiPlanStep::new(
                3,
                "Add Docker apt repository",
                "echo \"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo \"$VERSION_CODENAME\") stable\" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null && sudo apt-get update -y",
                "Register Docker stable repository in APT sources",
                false,
            ),
            AiPlanStep::new(
                4,
                "Install Docker CE & Compose",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin",
                "Install Docker Engine daemon, CLI, and Compose plugin",
                false,
            ),
            AiPlanStep::new(
                5,
                "Enable Docker service",
                "sudo systemctl enable --now docker",
                "Enable and start Docker daemon via systemd",
                false,
            ),
            AiPlanStep::new(
                6,
                "Add user to docker group",
                "sudo usermod -aG docker $USER",
                "Allow non-root user execution of docker commands",
                false,
            ),
            AiPlanStep::new(
                7,
                "Verify Docker installation",
                "docker --version && docker compose version",
                "Check Docker daemon and Compose plugin versions",
                false,
            ),
        ];

        AiExecutionPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            goal: goal.to_string(),
            summary: "Install and configure Docker Engine, Containerd, and Docker Compose plugin."
                .to_string(),
            host_id: target_host.map(|h| h.to_string()),
            host_label,
            requirements,
            steps,
            source: default_plan_source(),
            created_at: now,
            estimated_time: Some("~6 mins".to_string()),
        }
    } else if goal_lower.contains("node")
        || goal_lower.contains("express")
        || goal_lower.contains("next")
        || goal_lower.contains("react")
        || goal_lower.contains("fullstack")
    {
        let requirements = vec![
            "Node.js 20 LTS runtime".to_string(),
            "NPM package manager".to_string(),
            "PM2 process manager".to_string(),
            "Nginx reverse proxy".to_string(),
            "Build essential tools".to_string(),
        ];

        let steps = vec![
            AiPlanStep::new(
                1,
                "Update system packages",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get update -y && sudo apt-get install -y curl git build-essential",
                "Install build utilities, git, and curl",
                false,
            ),
            AiPlanStep::new(
                2,
                "Add NodeSource 20.x repo",
                "curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -",
                "Configure NodeSource APT repository for Node.js 20 LTS",
                false,
            ),
            AiPlanStep::new(
                3,
                "Install Node.js & NPM",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y nodejs",
                "Install Node.js 20 and NPM packages",
                false,
            ),
            AiPlanStep::new(
                4,
                "Install PM2 globally",
                "sudo npm install -g pm2",
                "Install PM2 production process supervisor",
                false,
            ),
            AiPlanStep::new(
                5,
                "Install Nginx web server",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y nginx && sudo systemctl enable --now nginx",
                "Install and start Nginx HTTP server for reverse proxying",
                false,
            ),
            AiPlanStep::new(
                6,
                "Verify runtime versions",
                "node -v && npm -v && pm2 -v && nginx -v",
                "Verify Node.js, PM2, and Nginx installations",
                false,
            ),
        ];

        AiExecutionPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            goal: goal.to_string(),
            summary:
                "Set up Node.js 20 LTS stack with PM2 process manager and Nginx reverse proxy."
                    .to_string(),
            host_id: target_host.map(|h| h.to_string()),
            host_label,
            requirements,
            steps,
            source: default_plan_source(),
            created_at: now,
            estimated_time: Some("~8 mins".to_string()),
        }
    } else if goal_lower.contains("python")
        || goal_lower.contains("django")
        || goal_lower.contains("fastapi")
        || goal_lower.contains("flask")
    {
        let requirements = vec![
            "Python 3 runtime & pip".to_string(),
            "Python virtualenv & venv module".to_string(),
            "Build utilities & libpq-dev (PostgreSQL client)".to_string(),
            "Nginx reverse proxy".to_string(),
        ];

        let steps = vec![
            AiPlanStep::new(
                1,
                "Install Python 3 and dependencies",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get update -y && sudo apt-get install -y python3 python3-pip python3-venv build-essential libpq-dev curl git",
                "Install Python 3, pip, venv module, and compilation headers",
                false,
            ),
            AiPlanStep::new(
                2,
                "Upgrade pip and virtualenv tools",
                "python3 -m pip install --upgrade pip setuptools wheel virtualenv --break-system-packages 2>/dev/null || python3 -m pip install --upgrade pip",
                "Update Python packaging tools",
                false,
            ),
            AiPlanStep::new(
                3,
                "Install Nginx web server",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get install -y nginx && sudo systemctl enable --now nginx",
                "Install and start Nginx server for ASGI/WSGI proxying",
                false,
            ),
            AiPlanStep::new(
                4,
                "Verify Python environment",
                "python3 --version && pip3 --version && nginx -v",
                "Verify installed Python, pip, and Nginx versions",
                false,
            ),
        ];

        AiExecutionPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            goal: goal.to_string(),
            summary: "Set up Python 3 production stack with virtualenv, build headers, and Nginx."
                .to_string(),
            host_id: target_host.map(|h| h.to_string()),
            host_label,
            requirements,
            steps,
            source: default_plan_source(),
            created_at: now,
            estimated_time: Some("~6 mins".to_string()),
        }
    } else if goal_lower.contains("hardening")
        || goal_lower.contains("ufw")
        || goal_lower.contains("firewall")
        || goal_lower.contains("security")
    {
        let requirements = vec![
            "UFW (Uncomplicated Firewall)".to_string(),
            "Fail2ban intrusion prevention daemon".to_string(),
            "SSH & HTTP firewall rules".to_string(),
        ];

        let steps = vec![
            AiPlanStep::new(
                1,
                "Install security packages",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get update -y && sudo apt-get install -y ufw fail2ban",
                "Install UFW firewall and Fail2ban service",
                false,
            ),
            AiPlanStep::new(
                2,
                "Configure firewall baseline rules",
                "sudo ufw default deny incoming && sudo ufw default allow outgoing && sudo ufw allow 22/tcp && sudo ufw allow 80/tcp && sudo ufw allow 443/tcp",
                "Set default deny incoming policy and allow SSH (22), HTTP (80), and HTTPS (443)",
                false,
            ),
            AiPlanStep::new(
                3,
                "Enable UFW firewall",
                "sudo ufw --force enable",
                "Activate firewall rules. Note: SSH port 22 is explicitly opened to avoid remote lockout.",
                true,
            ),
            AiPlanStep::new(
                4,
                "Enable Fail2ban service",
                "sudo systemctl enable --now fail2ban",
                "Start Fail2ban daemon to protect against brute-force attacks",
                false,
            ),
            AiPlanStep::new(
                5,
                "Verify security status",
                "sudo ufw status verbose && sudo systemctl status fail2ban --no-pager",
                "Inspect active firewall rules and Fail2ban service health",
                false,
            ),
        ];

        AiExecutionPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            goal: goal.to_string(),
            summary: "Apply Linux server hardening with UFW firewall baseline and Fail2ban intrusion protection.".to_string(),
            host_id: target_host.map(|h| h.to_string()),
            host_label,
            requirements,
            steps,
            source: default_plan_source(),
        created_at: now,
            estimated_time: Some("~4 mins".to_string()),
        }
    } else {
        let requirements = vec![
            "Bash shell environment".to_string(),
            "Sudo/root administrative privileges".to_string(),
            "System package manager (apt)".to_string(),
        ];

        let steps = vec![
            AiPlanStep::new(
                1,
                "Inspect system information",
                "uname -a && cat /etc/os-release",
                "Check operating system release and kernel details",
                false,
            ),
            AiPlanStep::new(
                2,
                "Update package index",
                "export DEBIAN_FRONTEND=noninteractive && sudo apt-get update -y",
                "Update local repository package lists",
                false,
            ),
            AiPlanStep::new(
                3,
                "Check disk and memory resources",
                "df -h && free -m",
                "Check storage partitions and RAM utilization before proceeding",
                false,
            ),
        ];

        AiExecutionPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            goal: goal.to_string(),
            summary: format!("Execution plan for: {goal}"),
            host_id: target_host.map(|h| h.to_string()),
            host_label,
            requirements,
            steps,
            source: default_plan_source(),
            created_at: now,
            estimated_time: Some("~3 mins".to_string()),
        }
    }
}

/// Generate a structured execution plan for a DevOps/Sysadmin goal.
/// Tries external LLM completion if configured, smoothly falling back to the template engine.
pub fn generate_plan(
    goal: &str,
    target_host: Option<&str>,
    hosted: bool,
) -> Result<AiExecutionPlan, CatermError> {
    if goal.trim().is_empty() {
        return Err(CatermError::Ai(AiError::Generic(
            "Goal cannot be empty".to_string(),
        )));
    }

    let host_label = resolve_host_label(target_host);

    if hosted {
        return call_llm_hosted(goal, target_host, host_label);
    }

    if let Ok(settings) = get_ai_settings()
        && let Some(plan) = call_llm_if_available(&settings, goal, target_host, host_label.clone())
    {
        return Ok(plan);
    }

    Ok(build_template_plan(goal, target_host, host_label))
}

/// Execute a single plan step command over SSH on the target host, recording the audit log.
/// One turn of a conversation with the assistant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiChatMessage {
    /// `system`, `user`, or `assistant`.
    pub role: String,
    pub content: String,
}

/// The assistant's answer to a turn. `steps` is empty for as long as the assistant is still
/// asking clarifying questions — it only proposes commands once `ready` is true.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChatReply {
    pub reply: String,
    pub ready: bool,
    pub steps: Vec<AiPlanStep>,
}

const CHAT_SYSTEM_PROMPT: &str = "You are CAFramework's Linux DevOps assistant, embedded in an SSH client. The user will describe something they want done on a remote server.

Your job has two phases.
PHASE 1 - CLARIFY: Ask short, concrete questions until you know enough to act. Ask about distribution and version, target versions of the software, whether sudo is available, ports, data directories, and anything destructive. Ask only what you genuinely need; never ask more than three questions in one turn. While you are in this phase, `ready` MUST be false and `steps` MUST be an empty array.
PHASE 2 - PROPOSE: Once the user has answered and explicitly agrees to proceed, set `ready` to true and fill `steps` with the exact shell commands, in order.

Commands run non-interactively over SSH, so they must not prompt: use flags like -y, set DEBIAN_FRONTEND=noninteractive, and never launch an editor or pager. Mark anything that deletes data, overwrites config, or restarts a service as dangerous.

Reply with ONLY a JSON object, no prose outside it, no markdown fence:
{\"reply\": \"what you say to the user, in the user's language\", \"ready\": false, \"steps\": [{\"step\": 1, \"title\": \"...\", \"command\": \"...\", \"description\": \"...\", \"is_dangerous\": false}]}";

/// Holds a conversation with the configured LLM.
///
/// Unlike [`generate_plan`], this has no heuristic fallback: a chat with no model behind it
/// would be a fiction, so a missing or unreachable endpoint is reported as an error the UI can
/// show instead of silently inventing an answer.
pub fn chat(
    messages: Vec<AiChatMessage>,
    host_label: Option<&str>,
    hosted: bool,
) -> Result<AiChatReply, CatermError> {
    if messages.is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "Percakapan kosong".to_string(),
        )));
    }

    let mut system = CHAT_SYSTEM_PROMPT.to_string();
    if let Some(label) = host_label.filter(|l| !l.trim().is_empty()) {
        system.push_str(&format!(
            "

The commands will run on the host the user calls \"{label}\"."
        ));
    }

    let mut payload_messages = vec![serde_json::json!({ "role": "system", "content": system })];
    for message in &messages {
        // Anything that is not a recognised role is treated as the user speaking: a malformed
        // role would otherwise be rejected by the endpoint and lose the whole conversation.
        let role = match message.role.as_str() {
            "assistant" => "assistant",
            "system" => "system",
            _ => "user",
        };
        payload_messages
            .push(serde_json::json!({ "role": role, "content": message.content.clone() }));
    }

    let content = if hosted {
        let payload = serde_json::json!({
            "messages": payload_messages,
            "temperature": 0.3,
            "stream": false
        });
        let body = crate::pro::ai_chat_completion(&payload)?;
        extract_message_content(&body.to_string()).ok_or_else(|| {
            CatermError::Ai(AiError::Generic(format!(
                "AI response does not match OpenAI chat/completions format: {}",
                body.to_string().chars().take(300).collect::<String>()
            )))
        })?
    } else {
        let settings = get_ai_settings()?;
        let payload = serde_json::json!({
            "model": &settings.model,
            "messages": payload_messages,
            "temperature": 0.3,
            // Some OpenAI-compatible gateways (9Router among them) stream by default and answer
            // with `text/event-stream` unless told otherwise, which no plain JSON parse can read.
            "stream": false
        });
        post_chat_completion(&settings, payload, std::time::Duration::from_secs(120))?
    };

    Ok(parse_chat_reply(&content))
}

/// Turns the model's answer into a reply. A model that ignores the JSON contract and answers in
/// prose still produces a usable turn — its text becomes the reply and nothing is proposed —
/// rather than an error the user cannot act on.
fn parse_chat_reply(content: &str) -> AiChatReply {
    let stripped = strip_code_fence(content);

    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(stripped) else {
        return AiChatReply {
            reply: content.trim().to_string(),
            ready: false,
            steps: Vec::new(),
        };
    };

    let reply = parsed
        .get("reply")
        .and_then(|r| r.as_str())
        .unwrap_or_else(|| content.trim())
        .to_string();

    let steps = parse_plan_steps(&parsed);
    // `ready` without steps is meaningless, and steps without `ready` would execute something
    // the user never agreed to — both are treated as "still talking".
    let ready = parsed
        .get("ready")
        .and_then(|r| r.as_bool())
        .unwrap_or(false)
        && !steps.is_empty();

    AiChatReply {
        reply,
        ready,
        steps: if ready { steps } else { Vec::new() },
    }
}

pub fn execute_plan_step(host_id: &str, command: &str) -> Result<AiExecutionResult, CatermError> {
    if host_id.trim().is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "host_id cannot be empty".to_string(),
        )));
    }
    if command.trim().is_empty() {
        return Err(CatermError::Validation(ValidationError::Generic(
            "command cannot be empty".to_string(),
        )));
    }

    if host_id == "local" || host_id == "__local__" {
        #[cfg(windows)]
        let mut cmd = std::process::Command::new("powershell.exe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.args(["-NoLogo", "-NonInteractive", "-Command", command]);
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        #[cfg(not(windows))]
        let mut cmd = std::process::Command::new("sh");
        #[cfg(not(windows))]
        cmd.args(["-c", command]);

        let output = cmd.output().map_err(|e| {
            CatermError::Ai(AiError::Generic(format!(
                "Failed to execute local command: {e}"
            )))
        })?;

        let exit_code = output.status.code().unwrap_or(0);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let summary = if success { "SUCCESS" } else { "FAILED" };

        let _ = crate::audit::log_event(
            "AI_AUTOMATION",
            Some(host_id),
            &format!("[AI-EXEC-LOCAL] Command: {command} Exit: {exit_code} Result: {summary}"),
        );

        return Ok(AiExecutionResult {
            step: 1,
            command: command.to_string(),
            success,
            exit_code,
            stdout,
            stderr,
            step_number: Some(1),
            duration_ms: None,
        });
    }

    // Runs on the host's pooled non-interactive session, NOT the terminal pane's session:
    // an automation step must not be able to stall (or steal output from) an open terminal.
    let (exit_code, stdout_buf, stderr_buf) = crate::ssh::with_exec_session(host_id, |sess| {
        let mut channel = sess.channel_session().map_err(|e| {
            CatermError::Ssh(SshError::Generic(format!(
                "Failed to open SSH channel: {e}"
            )))
        })?;

        channel.exec(command).map_err(|e| {
            CatermError::Ssh(SshError::Generic(format!("Failed to execute command: {e}")))
        })?;

        use std::io::Read;
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();

        let _ = channel.read_to_end(&mut stdout_buf);
        let _ = channel.stderr().read_to_end(&mut stderr_buf);
        let _ = channel.wait_close();

        Ok((channel.exit_status().unwrap_or(0), stdout_buf, stderr_buf))
    })?;

    let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
    let stderr = String::from_utf8_lossy(&stderr_buf).to_string();
    let success = exit_code == 0;
    let summary = if success { "SUCCESS" } else { "FAILED" };

    let _ = crate::audit::log_event(
        "AI_AUTOMATION",
        Some(host_id),
        &format!("[AI-EXEC] Command: {command} | Exit: {exit_code} | Result: {summary}"),
    );

    Ok(AiExecutionResult {
        step: 1,
        command: command.to_string(),
        success,
        exit_code,
        stdout,
        stderr,
        step_number: Some(1),
        duration_ms: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    struct TempDb(Connection, PathBuf);

    impl TempDb {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("waktu sistem sebelum epoch")
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("caframework_ai_test_{label}_{nanos:x}"));
            let conn =
                crate::db::open_encrypted(&dir, "test-ai-passphrase").expect("gagal buka db test");
            Self(conn, dir)
        }
    }

    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.1);
        }
    }

    #[test]
    fn extract_message_content_reads_a_plain_completion() {
        let body = r#"{"choices":[{"index":0,"message":{"role":"assistant","content":"pong"}}]}"#;
        assert_eq!(extract_message_content(body).as_deref(), Some("pong"));
    }

    #[test]
    fn extract_message_content_reassembles_a_streamed_completion() {
        // Shape a 9Router-style gateway returns when it streams regardless of `stream: false`.
        // Parsing this as one JSON object fails, which made every request look like a hard
        // failure while the HTTP status was 200.
        let body = concat!(
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\"}}]}

",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"pong. \"}}]}

",
            "data: {\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Send task.\"}}]}

",
            "data: [DONE]

"
        );
        assert_eq!(
            extract_message_content(body).as_deref(),
            Some("pong. Send task.")
        );
    }

    #[test]
    fn extract_message_content_rejects_a_body_with_no_text() {
        assert!(
            extract_message_content(
                "data: [DONE]

"
            )
            .is_none()
        );
        assert!(extract_message_content("not json at all").is_none());
        assert!(extract_message_content(r#"{"error":"API key required"}"#).is_none());
    }

    #[test]
    fn parse_chat_reply_withholds_steps_until_the_model_says_ready() {
        let asking = parse_chat_reply(
            r#"{"reply":"Ubuntu versi berapa?","ready":false,"steps":[{"step":1,"title":"x","command":"echo hi","description":"","is_dangerous":false}]}"#,
        );
        assert!(!asking.ready, "steps without ready must not be executable");
        assert!(asking.steps.is_empty());

        let ready = parse_chat_reply(
            r#"{"reply":"Siap.","ready":true,"steps":[{"step":1,"title":"x","command":"echo hi","description":"","is_dangerous":false}]}"#,
        );
        assert!(ready.ready);
        assert_eq!(ready.steps.len(), 1);
    }

    #[test]
    fn parse_chat_reply_falls_back_to_prose() {
        let reply = parse_chat_reply("Halo, mau setup apa?");
        assert_eq!(reply.reply, "Halo, mau setup apa?");
        assert!(!reply.ready);
        assert!(reply.steps.is_empty());
    }

    #[test]
    fn plan_payload_omits_model_for_hosted_and_sets_it_for_byo() {
        let hosted = plan_payload(None, "install docker");
        assert!(hosted.get("model").is_none());
        assert_eq!(hosted["messages"][1]["content"], "install docker");

        let byo = plan_payload(Some("gpt-4o"), "install docker");
        assert_eq!(byo["model"], "gpt-4o");
    }

    #[test]
    fn settings_default_and_roundtrip() {
        let db = TempDb::new("settings_roundtrip");
        let initial = get_ai_settings_in(&db.0).expect("failed to get default settings");
        assert_eq!(initial.provider, "custom");
        assert_eq!(initial.base_url, "http://100.76.150.46:3007/v1");
        assert_eq!(initial.model, "claude-3-5-sonnet");
        assert_eq!(initial.api_key, "");

        let updated = AiSettings {
            provider: "openai".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: "sk-test123456".to_string(),
            model: "gpt-4o".to_string(),
        };

        let saved = save_ai_settings_in(&db.0, updated.clone()).expect("failed to save settings");
        assert_eq!(saved, updated);

        let reloaded = get_ai_settings_in(&db.0).expect("failed to reload settings");
        assert_eq!(reloaded, updated);
    }

    #[test]
    fn generate_laravel_plan_has_seven_sequential_steps() {
        let plan = generate_plan(
            "Setup Laravel 11 on Ubuntu with PHP and Composer",
            None,
            false,
        )
        .expect("generate_plan failed");

        assert_eq!(
            plan.steps.len(),
            7,
            "Laravel plan must contain exactly 7 sequential steps"
        );
        assert!(plan.requirements.iter().any(|r| r.contains("PHP 8.3")));
        assert!(plan.requirements.iter().any(|r| r.contains("Composer v2")));
        assert!(plan.requirements.iter().any(|r| r.contains("MySQL server")));
        assert!(plan.requirements.iter().any(|r| r.contains("Node.js 20")));
        assert!(plan.requirements.iter().any(|r| r.contains("Git")));

        // Verify sequential steps
        assert!(plan.steps[0].command.contains("apt-get update"));
        assert!(plan.steps[1].command.contains("ppa:ondrej/php"));
        assert!(plan.steps[2].command.contains("php8.3"));
        assert!(plan.steps[2].command.contains("php8.3-mbstring"));
        assert!(plan.steps[3].command.contains("composer"));
        assert!(plan.steps[4].command.contains("mysql-server"));
        assert!(plan.steps[5].command.contains("setup_20.x"));
        assert!(plan.steps[6].command.contains("php -v"));
    }

    #[test]
    fn generate_docker_plan_has_docker_steps() {
        let plan = generate_plan("Install Docker and docker compose", None, false)
            .expect("generate_plan failed");

        assert!(plan.steps.len() >= 5);
        assert!(plan.steps.iter().any(|s| s.command.contains("docker-ce")));
        assert!(plan.requirements.iter().any(|r| r.contains("Docker")));
    }

    #[test]
    fn generate_node_plan_has_node_and_pm2() {
        let plan = generate_plan("Deploy Node fullstack app with Express", None, false)
            .expect("generate_plan failed");

        assert!(plan.steps.iter().any(|s| s.command.contains("setup_20.x")));
        assert!(plan.steps.iter().any(|s| s.command.contains("pm2")));
        assert!(plan.requirements.iter().any(|r| r.contains("Node.js 20")));
    }

    #[test]
    fn generate_python_plan_has_python_and_venv() {
        let plan = generate_plan("Setup Python Django backend", None, false)
            .expect("generate_plan failed");

        assert!(plan.steps.iter().any(|s| s.command.contains("python3")));
        assert!(plan.requirements.iter().any(|r| r.contains("Python 3")));
    }

    #[test]
    fn generate_ufw_plan_has_dangerous_step() {
        let plan = generate_plan("Linux hardening and configure UFW firewall", None, false)
            .expect("generate_plan failed");

        let enable_step = plan
            .steps
            .iter()
            .find(|s| s.command.contains("ufw --force enable"));
        assert!(enable_step.is_some());
        assert!(
            enable_step.unwrap().is_dangerous,
            "ufw enable step must be marked dangerous"
        );
    }

    #[test]
    fn generate_plan_rejects_empty_goal() {
        let err = generate_plan("   ", None, false);
        assert!(err.is_err());
    }

    #[test]
    fn execute_plan_step_rejects_empty_params() {
        assert!(execute_plan_step("", "ls").is_err());
        assert!(execute_plan_step("h1", "").is_err());
    }
}
