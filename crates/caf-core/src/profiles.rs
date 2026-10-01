//! Multi-profile management with PIN and roles.

use crate::db;
use crate::error::{AuthError, CatermError, DbError};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRecord {
    pub id: String,
    pub name: String,
    pub role: String, // 'owner' | 'partner' | 'child' | 'member'
    pub avatar: Option<String>,
    #[serde(skip_serializing)]
    pub pin_hash: Option<String>,
    pub has_pin: bool,
    pub rev: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub origin_device_id: String,
    pub owner_profile_id: String,
    pub visibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInput {
    pub id: Option<String>,
    pub name: String,
    pub role: String,
    pub avatar: Option<String>,
    pub pin: Option<String>,
}

pub fn list_profiles() -> Result<Vec<ProfileRecord>, CatermError> {
    let conn = db::open()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, role, avatar, pin_hash, rev, created_at, updated_at, deleted_at, origin_device_id, owner_profile_id, visibility 
             FROM profiles 
             WHERE deleted_at IS NULL
             ORDER BY created_at ASC",
        )
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let rows = stmt
        .query_map([], |row| {
            let pin_hash: Option<String> = row.get(4)?;
            let has_pin =
                pin_hash.is_some() && !pin_hash.as_ref().map(|s| s.is_empty()).unwrap_or(true);
            Ok(ProfileRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                role: row.get(2)?,
                avatar: row.get(3)?,
                pin_hash,
                has_pin,
                rev: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                deleted_at: row.get(8)?,
                origin_device_id: row.get(9)?,
                owner_profile_id: row.get(10)?,
                visibility: row.get(11)?,
            })
        })
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row.map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?);
    }

    Ok(list)
}

pub fn save_profile(
    input: ProfileInput,
    caller_profile_id: &str,
) -> Result<ProfileRecord, CatermError> {
    let mut conn = db::open()?;
    let tx = conn
        .transaction()
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let now = Utc::now().to_rfc3339();
    let device_id = crate::paths::device_id().unwrap_or_else(|_| "device-local".into());

    let pin_hash = if let Some(ref pin) = input.pin {
        if !pin.trim().is_empty() {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            Some(
                argon2
                    .hash_password(pin.as_bytes(), &salt)
                    .map_err(|e| {
                        CatermError::Auth(AuthError::Generic(format!("PIN hash failed: {e}")))
                    })?
                    .to_string(),
            )
        } else {
            None
        }
    } else {
        None
    };

    let (id, rev, created_at, _action) = if let Some(existing_id) = input.id {
        let existing: (i64, String, Option<String>) = tx
            .query_row(
                "SELECT rev, created_at, pin_hash FROM profiles WHERE id = ?1 AND deleted_at IS NULL",
                [&existing_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .map_err(|e| CatermError::Db(DbError::Generic(format!("Profile not found: {e}"))))?;

        let new_rev = existing.0 + 1;
        let final_pin_hash = pin_hash.or(existing.2);

        tx.execute(
            "UPDATE profiles SET name = ?1, role = ?2, avatar = ?3, pin_hash = ?4, rev = ?5, updated_at = ?6 WHERE id = ?7",
            rusqlite::params![input.name, input.role, input.avatar, final_pin_hash, new_rev, now, existing_id],
        )
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

        (existing_id, new_rev, existing.1, "update")
    } else {
        let new_id = Uuid::now_v7().to_string();
        let rev = 1;
        tx.execute(
            "INSERT INTO profiles (id, name, role, avatar, pin_hash, rev, created_at, updated_at, origin_device_id, owner_profile_id, visibility)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 'shared')",
            rusqlite::params![
                new_id,
                input.name,
                input.role,
                input.avatar,
                pin_hash,
                rev,
                now,
                now,
                device_id,
                caller_profile_id
            ],
        )
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

        (new_id, rev, now.clone(), "create")
    };

    tx.commit()
        .map_err(|e| CatermError::Db(DbError::Generic(e.to_string())))?;

    let has_pin = input.pin.is_some();
    Ok(ProfileRecord {
        id,
        name: input.name,
        role: input.role,
        avatar: input.avatar,
        pin_hash: None,
        has_pin,
        rev,
        created_at,
        updated_at: now,
        deleted_at: None,
        origin_device_id: device_id,
        owner_profile_id: caller_profile_id.to_string(),
        visibility: "shared".into(),
    })
}

pub fn verify_pin(profile_id: &str, pin: &str) -> Result<bool, CatermError> {
    let conn = db::open()?;
    let pin_hash: Option<String> = conn
        .query_row(
            "SELECT pin_hash FROM profiles WHERE id = ?1 AND deleted_at IS NULL",
            [profile_id],
            |r| r.get(0),
        )
        .map_err(|e| CatermError::Db(DbError::Generic(format!("Profile not found: {e}"))))?;

    let Some(hash_str) = pin_hash else {
        return Ok(true); // No PIN set
    };

    if hash_str.is_empty() {
        return Ok(true);
    }

    let parsed_hash = PasswordHash::new(&hash_str)
        .map_err(|e| CatermError::Auth(AuthError::Generic(format!("Corrupt PIN hash: {e}"))))?;

    let argon2 = Argon2::default();
    Ok(argon2.verify_password(pin.as_bytes(), &parsed_hash).is_ok())
}
