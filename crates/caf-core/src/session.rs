//! Core session and authorization state management.

use crate::error::{AuthError, CatermError};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Session {
    pub profile_id: String,
    pub role: String,
    pub started_at: u64,
    pub last_activity_at: u64,
}

impl Session {
    pub fn new(profile_id: impl Into<String>, role: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            profile_id: profile_id.into(),
            role: role.into(),
            started_at: now,
            last_activity_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_activity_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
}

#[derive(Debug, Default)]
pub struct SessionManager {
    active_session: Option<Session>,
}

static CURRENT_SESSION: std::sync::RwLock<Option<Session>> = std::sync::RwLock::new(None);

pub fn get_current_session() -> Result<Session, CatermError> {
    let lock = CURRENT_SESSION
        .read()
        .map_err(|_| CatermError::Auth(AuthError::Unauthorized))?;
    lock.clone()
        .ok_or_else(|| CatermError::Auth(AuthError::Unauthorized))
}

pub fn set_current_session(session: Session) -> Result<(), CatermError> {
    let mut lock = CURRENT_SESSION
        .write()
        .map_err(|_| CatermError::Auth(AuthError::Unauthorized))?;
    *lock = Some(session);
    Ok(())
}

pub fn clear_current_session() -> Result<(), CatermError> {
    let mut lock = CURRENT_SESSION
        .write()
        .map_err(|_| CatermError::Auth(AuthError::Unauthorized))?;
    *lock = None;
    Ok(())
}

impl SessionManager {
    pub fn set_session(&mut self, session: Session) {
        self.active_session = Some(session);
    }

    pub fn clear(&mut self) {
        self.active_session = None;
    }

    pub fn get(&self) -> Result<&Session, CatermError> {
        self.active_session
            .as_ref()
            .ok_or_else(|| CatermError::Auth(AuthError::Unauthorized))
    }

    pub fn get_mut(&mut self) -> Result<&mut Session, CatermError> {
        self.active_session
            .as_mut()
            .ok_or_else(|| CatermError::Auth(AuthError::Unauthorized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_lifecycle() {
        let mut sm = SessionManager::default();
        assert!(sm.get().is_err());

        sm.set_session(Session::new("prof_123", "owner"));
        let sess = sm.get().expect("valid session");
        assert_eq!(sess.profile_id, "prof_123");
        assert_eq!(sess.role, "owner");

        sm.clear();
        assert!(sm.get().is_err());
    }
}
