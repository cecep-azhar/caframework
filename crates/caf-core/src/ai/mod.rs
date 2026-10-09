//! Master AI Routing, Custom Context & Persistent Habit Memory Engine for CAFramework.
//! Combines custom personas, zero-knowledge SQLite/FTS5 habit memory, multi-provider model routing,
//! privacy scrubbing, custom skills catalog, and automated execution plans.

pub mod context_builder;
pub mod db;
pub mod dispatcher;
pub mod habit_learner;
pub mod legacy;
pub mod models;
pub mod scrubber;
pub mod skills;

pub use context_builder::*;
pub use db::*;
pub use dispatcher::*;
pub use habit_learner::*;
pub use legacy::*;
pub use models::*;
pub use scrubber::*;
pub use skills::*;

use crate::error::CafError;

// ---------------------------------------------------------------------------
// High-Level Public API for Tauri IPC Commands
// ---------------------------------------------------------------------------

pub fn get_all_providers() -> Result<Vec<AiProviderSummary>, CafError> {
    let conn = crate::db::open()?;
    let configs = db::get_providers(&conn)?;
    Ok(configs
        .into_iter()
        .map(|c| AiProviderSummary {
            id: c.id,
            name: c.name,
            provider_type: c.provider_type,
            base_url: c.base_url,
            default_model: c.default_model,
            is_active: c.is_active,
            has_api_key: c
                .api_key
                .as_ref()
                .map(|k| !k.trim().is_empty())
                .unwrap_or(false),
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
        .collect())
}

pub fn save_ai_provider(provider: AiProviderConfig) -> Result<(), CafError> {
    let conn = crate::db::open()?;
    db::save_provider(&conn, &provider)
}

pub fn delete_ai_provider(id: &str) -> Result<(), CafError> {
    let conn = crate::db::open()?;
    db::delete_provider(&conn, id)
}

pub fn get_task_routing_matrix() -> Result<Vec<TaskRouteRule>, CafError> {
    let conn = crate::db::open()?;
    db::get_routing_matrix(&conn)
}

pub fn save_task_routing_rule(rule: TaskRouteRule) -> Result<(), CafError> {
    let conn = crate::db::open()?;
    db::save_routing_rule(&conn, &rule)
}

pub fn get_user_personas() -> Result<Vec<SystemPersona>, CafError> {
    let conn = crate::db::open()?;
    db::get_personas(&conn)
}

pub fn save_user_persona(persona: SystemPersona) -> Result<(), CafError> {
    let conn = crate::db::open()?;
    db::save_persona(&conn, &persona)
}

pub fn delete_user_persona(id: &str) -> Result<(), CafError> {
    let conn = crate::db::open()?;
    db::delete_persona(&conn, id)
}

pub fn get_habit_memories() -> Result<Vec<HabitFact>, CafError> {
    let conn = crate::db::open()?;
    db::get_habits(&conn)
}

pub fn search_habit_memories(query: &str, limit: usize) -> Result<Vec<HabitFact>, CafError> {
    let conn = crate::db::open()?;
    db::search_relevant_habits(&conn, query, limit)
}

pub fn toggle_habit_pin_status(id: &str) -> Result<bool, CafError> {
    let conn = crate::db::open()?;
    db::toggle_habit_pin(&conn, id)
}

pub fn delete_habit_memory(id: &str) -> Result<(), CafError> {
    let conn = crate::db::open()?;
    db::delete_habit(&conn, id)
}

pub fn get_all_skills() -> Result<Vec<CustomSkill>, CafError> {
    let conn = crate::db::open()?;
    db::get_skills(&conn)
}

pub fn save_custom_skill(skill: CustomSkill) -> Result<(), CafError> {
    let conn = crate::db::open()?;
    db::save_skill(&conn, &skill)
}

pub fn delete_custom_skill(id: &str) -> Result<(), CafError> {
    let conn = crate::db::open()?;
    db::delete_skill(&conn, id)
}

pub fn dispatch_task(
    task_type: TaskType,
    user_query: &str,
    terminal_ctx: Option<TerminalContext>,
) -> Result<DispatchResult, CafError> {
    let conn = crate::db::open()?;
    let dispatcher = Dispatcher::new(&conn);
    dispatcher.dispatch(task_type, user_query, terminal_ctx.as_ref())
}

pub fn dispatch_task_with_skill(
    task_type: TaskType,
    user_query: &str,
    skill_name: Option<String>,
    terminal_ctx: Option<TerminalContext>,
) -> Result<DispatchResult, CafError> {
    let conn = crate::db::open()?;
    let dispatcher = Dispatcher::new(&conn);
    dispatcher.dispatch_with_skill(
        task_type,
        user_query,
        skill_name.as_deref(),
        terminal_ctx.as_ref(),
    )
}
