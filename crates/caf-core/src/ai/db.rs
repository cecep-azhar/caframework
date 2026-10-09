//! Database repository for AI providers, routing matrix, personas, and habit memory.

use super::models::{
    AiProviderConfig, HabitFact, ProviderType, SystemPersona, TaskRouteRule, TaskType,
};
use super::skills::CustomSkill;
use crate::error::{CafError, DbError};
use rusqlite::{Connection, params};

// ---------------------------------------------------------------------------
// 1. Providers
// ---------------------------------------------------------------------------

pub fn get_providers(conn: &Connection) -> Result<Vec<AiProviderConfig>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, provider_type, base_url, api_key_encrypted, default_model,
                    is_active, custom_headers_json, created_at, updated_at
             FROM ai_providers ORDER BY created_at ASC",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare providers query: {e}"))))?;

    let rows = stmt
        .query_map([], |row| {
            let ptype_str: String = row.get(2)?;
            let headers_json: String = row.get(7)?;
            let custom_headers = serde_json::from_str(&headers_json).ok();
            Ok(AiProviderConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: ProviderType::from_str_opt(&ptype_str)
                    .unwrap_or(ProviderType::OpenaiCompatible),
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                default_model: row.get(5)?,
                is_active: row.get::<_, i64>(6)? != 0,
                custom_headers,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute providers query: {e}"))))?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(item) = r {
            list.push(item);
        }
    }
    Ok(list)
}

pub fn get_provider(conn: &Connection, id: &str) -> Result<Option<AiProviderConfig>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, provider_type, base_url, api_key_encrypted, default_model,
                    is_active, custom_headers_json, created_at, updated_at
             FROM ai_providers WHERE id = ?1",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare provider query: {e}"))))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            let ptype_str: String = row.get(2)?;
            let headers_json: String = row.get(7)?;
            let custom_headers = serde_json::from_str(&headers_json).ok();
            Ok(AiProviderConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: ProviderType::from_str_opt(&ptype_str)
                    .unwrap_or(ProviderType::OpenaiCompatible),
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                default_model: row.get(5)?,
                is_active: row.get::<_, i64>(6)? != 0,
                custom_headers,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute provider query: {e}"))))?;

    match rows.next() {
        Some(Ok(cfg)) => Ok(Some(cfg)),
        _ => Ok(None),
    }
}

pub fn save_provider(conn: &Connection, provider: &AiProviderConfig) -> Result<(), CafError> {
    let headers_json = match &provider.custom_headers {
        Some(h) => serde_json::to_string(h).unwrap_or_else(|_| "{}".to_string()),
        None => "{}".to_string(),
    };

    conn.execute(
        "INSERT INTO ai_providers (
            id, name, provider_type, base_url, api_key_encrypted, default_model,
            is_active, custom_headers_json, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, strftime('%s', 'now'), strftime('%s', 'now'))
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            provider_type = excluded.provider_type,
            base_url = excluded.base_url,
            api_key_encrypted = coalesce(excluded.api_key_encrypted, ai_providers.api_key_encrypted),
            default_model = excluded.default_model,
            is_active = excluded.is_active,
            custom_headers_json = excluded.custom_headers_json,
            updated_at = strftime('%s', 'now');",
        params![
            provider.id,
            provider.name,
            provider.provider_type.as_str(),
            provider.base_url,
            provider.api_key,
            provider.default_model,
            if provider.is_active { 1 } else { 0 },
            headers_json,
        ],
    )
    .map_err(|e| CafError::Db(DbError::Generic(format!("save provider: {e}"))))?;

    Ok(())
}

pub fn delete_provider(conn: &Connection, id: &str) -> Result<(), CafError> {
    conn.execute("DELETE FROM ai_providers WHERE id = ?1", params![id])
        .map_err(|e| CafError::Db(DbError::Generic(format!("delete provider: {e}"))))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. Routing Matrix
// ---------------------------------------------------------------------------

pub fn get_routing_matrix(conn: &Connection) -> Result<Vec<TaskRouteRule>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT task_type, primary_provider_id, primary_model,
                    fallback_provider_id, fallback_model, temperature, max_tokens, system_persona_id
             FROM ai_routing_matrix ORDER BY task_type ASC",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare matrix query: {e}"))))?;

    let rows = stmt
        .query_map([], |row| {
            let task_str: String = row.get(0)?;
            Ok(TaskRouteRule {
                task_type: TaskType::from_str_opt(&task_str).unwrap_or(TaskType::Chat),
                primary_provider_id: row.get(1)?,
                primary_model: row.get(2)?,
                fallback_provider_id: row.get(3)?,
                fallback_model: row.get(4)?,
                temperature: row.get(5)?,
                max_tokens: row.get(6)?,
                system_persona_id: row.get(7)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute matrix query: {e}"))))?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(item) = r {
            list.push(item);
        }
    }
    Ok(list)
}

pub fn get_route_rule(
    conn: &Connection,
    task_type: TaskType,
) -> Result<Option<TaskRouteRule>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT task_type, primary_provider_id, primary_model,
                    fallback_provider_id, fallback_model, temperature, max_tokens, system_persona_id
             FROM ai_routing_matrix WHERE task_type = ?1",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare route rule query: {e}"))))?;

    let mut rows = stmt
        .query_map(params![task_type.as_str()], |row| {
            let task_str: String = row.get(0)?;
            Ok(TaskRouteRule {
                task_type: TaskType::from_str_opt(&task_str).unwrap_or(TaskType::Chat),
                primary_provider_id: row.get(1)?,
                primary_model: row.get(2)?,
                fallback_provider_id: row.get(3)?,
                fallback_model: row.get(4)?,
                temperature: row.get(5)?,
                max_tokens: row.get(6)?,
                system_persona_id: row.get(7)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute route rule query: {e}"))))?;

    match rows.next() {
        Some(Ok(rule)) => Ok(Some(rule)),
        _ => Ok(None),
    }
}

pub fn save_routing_rule(conn: &Connection, rule: &TaskRouteRule) -> Result<(), CafError> {
    conn.execute(
        "INSERT OR REPLACE INTO ai_routing_matrix (
            task_type, primary_provider_id, primary_model,
            fallback_provider_id, fallback_model, temperature, max_tokens, system_persona_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
        params![
            rule.task_type.as_str(),
            rule.primary_provider_id,
            rule.primary_model,
            rule.fallback_provider_id,
            rule.fallback_model,
            rule.temperature,
            rule.max_tokens,
            rule.system_persona_id,
        ],
    )
    .map_err(|e| CafError::Db(DbError::Generic(format!("save routing rule: {e}"))))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. System Personas
// ---------------------------------------------------------------------------

pub fn get_personas(conn: &Connection) -> Result<Vec<SystemPersona>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, system_prompt, custom_rules_json,
                    environment_constraints, is_global_default, created_at, updated_at
             FROM ai_user_personas ORDER BY is_global_default DESC, title ASC",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare personas query: {e}"))))?;

    let rows = stmt
        .query_map([], |row| {
            let rules_json: String = row.get(4)?;
            let custom_rules: Vec<String> = serde_json::from_str(&rules_json).unwrap_or_default();
            Ok(SystemPersona {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                system_prompt: row.get(3)?,
                custom_rules,
                environment_constraints: row.get(5)?,
                is_global_default: row.get::<_, i64>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute personas query: {e}"))))?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(item) = r {
            list.push(item);
        }
    }
    Ok(list)
}

pub fn get_persona(conn: &Connection, id: &str) -> Result<Option<SystemPersona>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, system_prompt, custom_rules_json,
                    environment_constraints, is_global_default, created_at, updated_at
             FROM ai_user_personas WHERE id = ?1",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare persona query: {e}"))))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            let rules_json: String = row.get(4)?;
            let custom_rules: Vec<String> = serde_json::from_str(&rules_json).unwrap_or_default();
            Ok(SystemPersona {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                system_prompt: row.get(3)?,
                custom_rules,
                environment_constraints: row.get(5)?,
                is_global_default: row.get::<_, i64>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute persona query: {e}"))))?;

    match rows.next() {
        Some(Ok(p)) => Ok(Some(p)),
        _ => Ok(None),
    }
}

pub fn get_default_persona(conn: &Connection) -> Result<Option<SystemPersona>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, system_prompt, custom_rules_json,
                    environment_constraints, is_global_default, created_at, updated_at
             FROM ai_user_personas WHERE is_global_default = 1 LIMIT 1",
        )
        .map_err(|e| {
            CafError::Db(DbError::Generic(format!(
                "prepare default persona query: {e}"
            )))
        })?;

    let mut rows = stmt
        .query_map([], |row| {
            let rules_json: String = row.get(4)?;
            let custom_rules: Vec<String> = serde_json::from_str(&rules_json).unwrap_or_default();
            Ok(SystemPersona {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                system_prompt: row.get(3)?,
                custom_rules,
                environment_constraints: row.get(5)?,
                is_global_default: row.get::<_, i64>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| {
            CafError::Db(DbError::Generic(format!(
                "execute default persona query: {e}"
            )))
        })?;

    match rows.next() {
        Some(Ok(p)) => Ok(Some(p)),
        _ => Ok(None),
    }
}

pub fn save_persona(conn: &Connection, persona: &SystemPersona) -> Result<(), CafError> {
    let rules_json =
        serde_json::to_string(&persona.custom_rules).unwrap_or_else(|_| "[]".to_string());

    if persona.is_global_default {
        let _ = conn.execute("UPDATE ai_user_personas SET is_global_default = 0", []);
    }

    conn.execute(
        "INSERT INTO ai_user_personas (
            id, title, description, system_prompt, custom_rules_json,
            environment_constraints, is_global_default, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%s', 'now'), strftime('%s', 'now'))
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            description = excluded.description,
            system_prompt = excluded.system_prompt,
            custom_rules_json = excluded.custom_rules_json,
            environment_constraints = excluded.environment_constraints,
            is_global_default = excluded.is_global_default,
            updated_at = strftime('%s', 'now');",
        params![
            persona.id,
            persona.title,
            persona.description,
            persona.system_prompt,
            rules_json,
            persona.environment_constraints,
            if persona.is_global_default { 1 } else { 0 },
        ],
    )
    .map_err(|e| CafError::Db(DbError::Generic(format!("save persona: {e}"))))?;

    Ok(())
}

pub fn delete_persona(conn: &Connection, id: &str) -> Result<(), CafError> {
    conn.execute("DELETE FROM ai_user_personas WHERE id = ?1", params![id])
        .map_err(|e| CafError::Db(DbError::Generic(format!("delete persona: {e}"))))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. Habit Memory & FTS5 Retrieval
// ---------------------------------------------------------------------------

pub fn get_habits(conn: &Connection) -> Result<Vec<HabitFact>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, category, key_tag, fact_content, source_context,
                    confidence_score, occurrence_count, is_pinned, is_active,
                    created_at, last_accessed_at
             FROM ai_habits
             ORDER BY is_pinned DESC, confidence_score DESC, occurrence_count DESC",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare habits query: {e}"))))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(HabitFact {
                id: row.get(0)?,
                category: row.get(1)?,
                key_tag: row.get(2)?,
                fact_content: row.get(3)?,
                source_context: row.get(4)?,
                confidence_score: row.get(5)?,
                occurrence_count: row.get(6)?,
                is_pinned: row.get::<_, i64>(7)? != 0,
                is_active: row.get::<_, i64>(8)? != 0,
                created_at: row.get(9)?,
                last_accessed_at: row.get(10)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute habits query: {e}"))))?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(item) = r {
            list.push(item);
        }
    }
    Ok(list)
}

pub fn save_habit(conn: &Connection, habit: &HabitFact) -> Result<(), CafError> {
    conn.execute(
        "INSERT INTO ai_habits (
            id, category, key_tag, fact_content, source_context,
            confidence_score, occurrence_count, is_pinned, is_active,
            created_at, last_accessed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, strftime('%s', 'now'), strftime('%s', 'now'))
        ON CONFLICT(id) DO UPDATE SET
            category = excluded.category,
            key_tag = excluded.key_tag,
            fact_content = excluded.fact_content,
            source_context = excluded.source_context,
            confidence_score = excluded.confidence_score,
            occurrence_count = excluded.occurrence_count,
            is_pinned = excluded.is_pinned,
            is_active = excluded.is_active,
            last_accessed_at = strftime('%s', 'now');",
        params![
            habit.id,
            habit.category,
            habit.key_tag,
            habit.fact_content,
            habit.source_context,
            habit.confidence_score,
            habit.occurrence_count,
            if habit.is_pinned { 1 } else { 0 },
            if habit.is_active { 1 } else { 0 },
        ],
    )
    .map_err(|e| CafError::Db(DbError::Generic(format!("save habit: {e}"))))?;

    Ok(())
}

pub fn toggle_habit_pin(conn: &Connection, id: &str) -> Result<bool, CafError> {
    let current_pinned: bool = conn
        .query_row(
            "SELECT is_pinned FROM ai_habits WHERE id = ?1",
            params![id],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("get habit pin status: {e}"))))?;

    let new_pinned = !current_pinned;
    conn.execute(
        "UPDATE ai_habits SET is_pinned = ?1 WHERE id = ?2",
        params![if new_pinned { 1 } else { 0 }, id],
    )
    .map_err(|e| CafError::Db(DbError::Generic(format!("update habit pin status: {e}"))))?;

    Ok(new_pinned)
}

pub fn toggle_habit_active(conn: &Connection, id: &str) -> Result<bool, CafError> {
    let current_active: bool = conn
        .query_row(
            "SELECT is_active FROM ai_habits WHERE id = ?1",
            params![id],
            |r| Ok(r.get::<_, i64>(0)? != 0),
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("get habit active status: {e}"))))?;

    let new_active = !current_active;
    conn.execute(
        "UPDATE ai_habits SET is_active = ?1 WHERE id = ?2",
        params![if new_active { 1 } else { 0 }, id],
    )
    .map_err(|e| CafError::Db(DbError::Generic(format!("update habit active status: {e}"))))?;

    Ok(new_active)
}

pub fn delete_habit(conn: &Connection, id: &str) -> Result<(), CafError> {
    conn.execute("DELETE FROM ai_habits WHERE id = ?1", params![id])
        .map_err(|e| CafError::Db(DbError::Generic(format!("delete habit: {e}"))))?;
    Ok(())
}

/// Search relevant habits via FTS5 full-text index with graceful token sanitization.
pub fn search_relevant_habits(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<HabitFact>, CafError> {
    // Sanitize query tokens for FTS5 (keep alphanumerics, words with prefix match)
    let sanitized_tokens: Vec<String> = query
        .split_whitespace()
        .filter_map(|w| {
            let cleaned: String = w
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if cleaned.len() >= 2 {
                Some(format!("\"{cleaned}\"*"))
            } else {
                None
            }
        })
        .collect();

    if sanitized_tokens.is_empty() {
        // Fallback: return top active habits (pinned first)
        let mut stmt = conn
            .prepare(
                "SELECT id, category, key_tag, fact_content, source_context,
                        confidence_score, occurrence_count, is_pinned, is_active,
                        created_at, last_accessed_at
                 FROM ai_habits
                 WHERE is_active = 1
                 ORDER BY is_pinned DESC, confidence_score DESC
                 LIMIT ?1",
            )
            .map_err(|e| {
                CafError::Db(DbError::Generic(format!(
                    "prepare default habits query: {e}"
                )))
            })?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(HabitFact {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    key_tag: row.get(2)?,
                    fact_content: row.get(3)?,
                    source_context: row.get(4)?,
                    confidence_score: row.get(5)?,
                    occurrence_count: row.get(6)?,
                    is_pinned: row.get::<_, i64>(7)? != 0,
                    is_active: row.get::<_, i64>(8)? != 0,
                    created_at: row.get(9)?,
                    last_accessed_at: row.get(10)?,
                })
            })
            .map_err(|e| {
                CafError::Db(DbError::Generic(format!(
                    "execute default habits query: {e}"
                )))
            })?;

        let mut results = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                results.push(item);
            }
        }
        return Ok(results);
    }

    let fts_match_query = sanitized_tokens.join(" OR ");

    let mut stmt = conn
        .prepare(
            "SELECT h.id, h.category, h.key_tag, h.fact_content, h.source_context,
                    h.confidence_score, h.occurrence_count, h.is_pinned, h.is_active,
                    h.created_at, h.last_accessed_at
             FROM ai_habits h
             JOIN ai_habits_fts fts ON h.id = fts.id
             WHERE ai_habits_fts MATCH ?1 AND h.is_active = 1
             ORDER BY h.is_pinned DESC, rank
             LIMIT ?2",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare fts query: {e}"))))?;

    let rows = stmt.query_map(params![fts_match_query, limit as i64], |row| {
        Ok(HabitFact {
            id: row.get(0)?,
            category: row.get(1)?,
            key_tag: row.get(2)?,
            fact_content: row.get(3)?,
            source_context: row.get(4)?,
            confidence_score: row.get(5)?,
            occurrence_count: row.get(6)?,
            is_pinned: row.get::<_, i64>(7)? != 0,
            is_active: row.get::<_, i64>(8)? != 0,
            created_at: row.get(9)?,
            last_accessed_at: row.get(10)?,
        })
    });

    match rows {
        Ok(mapped) => {
            let mut results = Vec::new();
            for r in mapped {
                if let Ok(item) = r {
                    results.push(item);
                }
            }
            if !results.is_empty() {
                return Ok(results);
            }
        }
        Err(_) => {
            // FTS error fallback
        }
    }

    // Secondary fallback: LIKE matching on fact_content or key_tag
    let fallback_token = format!("%{}%", query.trim());
    let mut stmt = conn
        .prepare(
            "SELECT id, category, key_tag, fact_content, source_context,
                    confidence_score, occurrence_count, is_pinned, is_active,
                    created_at, last_accessed_at
             FROM ai_habits
             WHERE is_active = 1 AND (fact_content LIKE ?1 OR key_tag LIKE ?1)
             ORDER BY is_pinned DESC, confidence_score DESC
             LIMIT ?2",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare like query: {e}"))))?;

    let rows = stmt
        .query_map(params![fallback_token, limit as i64], |row| {
            Ok(HabitFact {
                id: row.get(0)?,
                category: row.get(1)?,
                key_tag: row.get(2)?,
                fact_content: row.get(3)?,
                source_context: row.get(4)?,
                confidence_score: row.get(5)?,
                occurrence_count: row.get(6)?,
                is_pinned: row.get::<_, i64>(7)? != 0,
                is_active: row.get::<_, i64>(8)? != 0,
                created_at: row.get(9)?,
                last_accessed_at: row.get(10)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute like query: {e}"))))?;

    let mut results = Vec::new();
    for r in rows {
        if let Ok(item) = r {
            results.push(item);
        }
    }
    Ok(results)
}

// ---------------------------------------------------------------------------
// 5. Custom Skills
// ---------------------------------------------------------------------------

pub fn get_skills(conn: &Connection) -> Result<Vec<CustomSkill>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, title, description, category, triggers_json,
                    preferred_model_id, system_instructions, allowed_tools_json,
                    is_builtin, is_enabled, created_at
             FROM ai_custom_skills ORDER BY is_builtin DESC, name ASC",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare skills query: {e}"))))?;

    let rows = stmt
        .query_map([], |row| {
            let triggers_json: String = row.get(5)?;
            let tools_json: String = row.get(8)?;
            let triggers: Vec<String> = serde_json::from_str(&triggers_json).unwrap_or_default();
            let allowed_tools: Vec<String> = serde_json::from_str(&tools_json).unwrap_or_default();

            Ok(CustomSkill {
                id: row.get(0)?,
                name: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                triggers,
                preferred_model_id: row.get(6)?,
                system_instructions: row.get(7)?,
                allowed_tools,
                is_builtin: row.get::<_, i64>(9)? != 0,
                is_enabled: row.get::<_, i64>(10)? != 0,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute skills query: {e}"))))?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(item) = r {
            list.push(item);
        }
    }
    Ok(list)
}

pub fn get_skill_by_name(
    conn: &Connection,
    name: &str,
) -> Result<Option<CustomSkill>, CafError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, title, description, category, triggers_json,
                    preferred_model_id, system_instructions, allowed_tools_json,
                    is_builtin, is_enabled, created_at
             FROM ai_custom_skills WHERE name = ?1",
        )
        .map_err(|e| CafError::Db(DbError::Generic(format!("prepare skill query: {e}"))))?;

    let mut rows = stmt
        .query_map(params![name], |row| {
            let triggers_json: String = row.get(5)?;
            let tools_json: String = row.get(8)?;
            let triggers: Vec<String> = serde_json::from_str(&triggers_json).unwrap_or_default();
            let allowed_tools: Vec<String> = serde_json::from_str(&tools_json).unwrap_or_default();

            Ok(CustomSkill {
                id: row.get(0)?,
                name: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                category: row.get(4)?,
                triggers,
                preferred_model_id: row.get(6)?,
                system_instructions: row.get(7)?,
                allowed_tools,
                is_builtin: row.get::<_, i64>(9)? != 0,
                is_enabled: row.get::<_, i64>(10)? != 0,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| CafError::Db(DbError::Generic(format!("execute skill query: {e}"))))?;

    match rows.next() {
        Some(Ok(s)) => Ok(Some(s)),
        _ => Ok(None),
    }
}

pub fn save_skill(conn: &Connection, skill: &CustomSkill) -> Result<(), CafError> {
    let triggers_json = serde_json::to_string(&skill.triggers).unwrap_or_else(|_| "[]".to_string());
    let tools_json =
        serde_json::to_string(&skill.allowed_tools).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT INTO ai_custom_skills (
            id, name, title, description, category, triggers_json,
            preferred_model_id, system_instructions, allowed_tools_json,
            is_builtin, is_enabled, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, strftime('%s', 'now'))
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            title = excluded.title,
            description = excluded.description,
            category = excluded.category,
            triggers_json = excluded.triggers_json,
            preferred_model_id = excluded.preferred_model_id,
            system_instructions = excluded.system_instructions,
            allowed_tools_json = excluded.allowed_tools_json,
            is_enabled = excluded.is_enabled;",
        params![
            skill.id,
            skill.name,
            skill.title,
            skill.description,
            skill.category,
            triggers_json,
            skill.preferred_model_id,
            skill.system_instructions,
            tools_json,
            if skill.is_builtin { 1 } else { 0 },
            if skill.is_enabled { 1 } else { 0 },
        ],
    )
    .map_err(|e| CafError::Db(DbError::Generic(format!("save custom skill: {e}"))))?;

    Ok(())
}

pub fn delete_skill(conn: &Connection, id: &str) -> Result<(), CafError> {
    conn.execute(
        "DELETE FROM ai_custom_skills WHERE id = ?1 AND is_builtin = 0",
        params![id],
    )
    .map_err(|e| CafError::Db(DbError::Generic(format!("delete custom skill: {e}"))))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_encrypted;
    use std::path::PathBuf;

    struct TempDb(PathBuf);
    impl TempDb {
        fn new(name: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            Self(std::env::temp_dir().join(format!("caterm_ai_db_{name}_{nanos:x}")))
        }
    }
    impl Drop for TempDb {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn test_fts5_habit_search() {
        let t = TempDb::new("fts_habits");
        let conn = open_encrypted(&t.0, "test-ai-key").expect("open db");

        let habit = HabitFact {
            id: uuid::Uuid::new_v4().to_string(),
            category: "cli_preference".to_string(),
            key_tag: "docker_runtime".to_string(),
            fact_content: "Prefers podman over docker, uses rootless mode".to_string(),
            source_context: Some("terminal on server-01".to_string()),
            confidence_score: 0.95,
            occurrence_count: 3,
            is_pinned: true,
            is_active: true,
            created_at: 0,
            last_accessed_at: 0,
        };

        save_habit(&conn, &habit).expect("save habit");

        let found = search_relevant_habits(&conn, "podman docker", 5).expect("search");
        assert!(!found.is_empty(), "Should find podman habit via FTS5");
        assert_eq!(
            found.first().map(|h| h.key_tag.as_str()),
            Some("docker_runtime")
        );
    }
}
