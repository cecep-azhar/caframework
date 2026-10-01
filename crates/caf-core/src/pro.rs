//! CAFramework Pro client: account session, device activation and offline license verification.
//!
//! Server contract: `gcc/apps/gcc/internal/caframeworkpro/API.md`. In short:
//! - The Pro account (email + password) is separate from the vault master password. The
//!   vault stays zero-knowledge; nothing sent to the server can decrypt it.
//! - Signing in on the lock screen happens *before* the vault is unlocked, so the session is
//!   held in memory ([`commit_pending`] persists it once the vault opens).
//! - Entitlement is decided offline from an Ed25519-signed token the server re-issues on each
//!   heartbeat (valid ≤ 14 days). The public key is compiled in; the client can verify but
//!   never mint tokens.
//!
//! - Devices are counted per account: a Pro account may activate 5, a free account that is a
//!   member of someone's team 1. Membership grants the `team` tier while the owner is entitled.
//!
//! Errors are `CatermError::Pro` whose message is the server's stable code
//! (`INVALID_CREDENTIALS`, `DEVICE_LIMIT_EXCEEDED`, `NETWORK`, ...) so the UI can translate it.

use crate::error::{CatermError, DbError, ProError};
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Duration;

const DEFAULT_API_BASE: &str = "https://caframework.fathforce.com/api/pro/v1";
const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// NTP jitter we tolerate before treating a clock that went backwards as tampering.
const CLOCK_DRIFT_TOLERANCE_SECS: i64 = 300;
const HTTP_TIMEOUT: Duration = Duration::from_secs(15);

fn pro_err(code: impl Into<String>) -> CatermError {
    CatermError::Pro(ProError::Generic(code.into()))
}

fn db_err(e: rusqlite::Error) -> CatermError {
    CatermError::Db(DbError::Generic(format!("pro_state: {e}")))
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// `CATERM_PRO_API_BASE` at runtime (local GCC during development), else at build time, else
/// the product domain. Pointing it elsewhere cannot fake Pro: tokens still have to verify
/// against the compiled-in public key.
fn api_base() -> String {
    std::env::var("CATERM_PRO_API_BASE")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| option_env!("CATERM_PRO_API_BASE").map(str::to_string))
        .unwrap_or_else(|| DEFAULT_API_BASE.to_string())
        .trim_end_matches('/')
        .to_string()
}

// ---- Keys & tokens ---------------------------------------------------------------------------

/// Public key for a token's `key_version`, injected at build time from
/// `CATERM_PRO_LICENSE_PUBKEY_V<n>` (hex, made by GCC's `cmd/caframework-pro-keygen`). Old versions
/// stay listed after a rotation so tokens issued under them still verify until they expire.
/// Debug builds also accept `CATERM_PRO_LICENSE_PUBKEY` at runtime, for a local GCC.
fn public_key(version: u32) -> Option<VerifyingKey> {
    let compiled = match version {
        1 => option_env!("CATERM_PRO_LICENSE_PUBKEY_V1").or(Some(
            "8a1c49d8d6e8e6b2e25f821451c8d85b997331d2efa2d6d335e87535aa67f6a2",
        )),
        2 => option_env!("CATERM_PRO_LICENSE_PUBKEY_V2"),
        _ => None,
    };
    #[cfg(debug_assertions)]
    let runtime = std::env::var("CATERM_PRO_LICENSE_PUBKEY").ok();
    #[cfg(not(debug_assertions))]
    let runtime: Option<String> = None;

    let hex_key = compiled.map(str::to_string).or(runtime)?;
    let bytes: [u8; 32] = hex::decode(hex_key.trim()).ok()?.try_into().ok()?;
    VerifyingKey::from_bytes(&bytes).ok()
}

/// # Infallible: only reads keys compiled into this build; there is nothing to fail.
pub fn key_configured() -> bool {
    public_key(1).is_some()
}

#[derive(Debug, Clone, Deserialize)]
struct TokenPayload {
    hwid: String,
    tier: String,
    features: Vec<String>,
    exp: i64,
}

/// The signed token as the server sends it. `payload` is the exact signed string.
#[derive(Debug, Clone, Deserialize)]
struct SignedToken {
    payload: String,
    signature: String,
    key_version: u32,
}

fn verify_with(
    key: &VerifyingKey,
    payload: &str,
    signature_b64: &str,
) -> Result<TokenPayload, &'static str> {
    let sig_bytes = STANDARD
        .decode(signature_b64)
        .map_err(|_| "BAD_SIGNATURE")?;
    let signature = Signature::from_slice(&sig_bytes).map_err(|_| "BAD_SIGNATURE")?;
    key.verify(payload.as_bytes(), &signature)
        .map_err(|_| "BAD_SIGNATURE")?;
    serde_json::from_str(payload).map_err(|_| "BAD_PAYLOAD")
}

/// What this device may do right now, decided without the network.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Entitlement {
    /// Never activated on this device, or signed out.
    None,
    Valid {
        #[serde(rename = "expiresAt")]
        expires_at: i64,
        tier: String,
        features: Vec<String>,
    },
    /// Token ran out (trial or period ended, or offline for more than 14 days).
    Expired,
    /// The clock went backwards past the last time we saw: go online before trusting it.
    RevalidationRequired { reason: String },
    /// Signature, key or device mismatch — the stored token can't be trusted.
    Invalid { reason: String },
}

/// Pure decision, separated for tests: verifies the token and applies the rollback guard.
/// Returns the entitlement and the new monotonic "last seen" wall clock.
fn evaluate(
    token: Option<(&str, &str, u32)>,
    key_for: impl Fn(u32) -> Option<VerifyingKey>,
    hwid: &str,
    last_seen: i64,
    now: i64,
) -> (Entitlement, i64) {
    let Some((payload, signature, version)) = token else {
        return (Entitlement::None, last_seen);
    };
    let Some(key) = key_for(version) else {
        return (
            Entitlement::Invalid {
                reason: "KEY_NOT_CONFIGURED".into(),
            },
            last_seen,
        );
    };
    let claims = match verify_with(&key, payload, signature) {
        Ok(c) => c,
        Err(reason) => {
            return (
                Entitlement::Invalid {
                    reason: reason.into(),
                },
                last_seen,
            );
        }
    };
    if claims.hwid != hwid {
        // A database copied from another machine carries that machine's token.
        return (
            Entitlement::Invalid {
                reason: "OTHER_DEVICE".into(),
            },
            last_seen,
        );
    }
    if now < last_seen - CLOCK_DRIFT_TOLERANCE_SECS {
        return (
            Entitlement::RevalidationRequired {
                reason: "CLOCK_ROLLBACK".into(),
            },
            last_seen,
        );
    }
    let seen = now.max(last_seen);
    if now >= claims.exp {
        return (Entitlement::Expired, seen);
    }
    (
        Entitlement::Valid {
            expires_at: claims.exp,
            tier: claims.tier,
            features: claims.features,
        },
        seen,
    )
}

// ---- Device identity -------------------------------------------------------------------------

#[cfg(not(target_os = "android"))]
fn get_machine_uid() -> Result<String, CatermError> {
    machine_uid::get().map_err(|e| pro_err(format!("HWID_UNAVAILABLE: {e}")))
}

#[cfg(target_os = "android")]
fn get_machine_uid() -> Result<String, CatermError> {
    Ok("caframework-android-client".to_string())
}

/// Stable per-machine id: SHA-256 of the machine-id (MachineGuid / machine-id / IOPlatformUUID,
/// no admin rights needed) + OS. A clean OS reinstall yields a new id; the same limit
/// as every other software-only licence (licensing notes 7.2).
pub fn hwid() -> Result<String, CatermError> {
    let machine = get_machine_uid()?;
    let mut hasher = Sha256::new();
    hasher.update(b"caframework-pro-hwid-v1\0");
    hasher.update(machine.trim().as_bytes());
    hasher.update(b"\0");
    hasher.update(std::env::consts::OS.as_bytes());
    Ok(hex::encode(hasher.finalize()))
}

fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "CAFramework device".to_string())
}

fn device_body(hwid: &str) -> Value {
    json!({
        "hwid": hwid,
        "device_name": device_name(),
        "os": std::env::consts::OS,
        "client_version": CLIENT_VERSION,
    })
}

// ---- HTTP ------------------------------------------------------------------------------------

fn request(
    method: &str,
    path: &str,
    bearer: Option<&str>,
    body: Option<&Value>,
) -> Result<(u16, Value), CatermError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(HTTP_TIMEOUT))
        .http_status_as_error(false) // we need the JSON error code in 4xx bodies
        .build()
        .into();
    let url = format!("{}{path}", api_base());
    let result = match method {
        "GET" | "DELETE" => {
            let mut req = if method == "GET" {
                agent.get(&url)
            } else {
                agent.delete(&url)
            };
            if let Some(token) = bearer {
                req = req.header("Authorization", &format!("Bearer {token}"));
            }
            req.call()
        }
        _ => {
            let mut req = agent.post(&url).header("Content-Type", "application/json");
            if let Some(token) = bearer {
                req = req.header("Authorization", &format!("Bearer {token}"));
            }
            req.send_json(body.cloned().unwrap_or_else(|| json!({})))
        }
    };
    let mut response = result.map_err(|_| pro_err("NETWORK"))?;
    let status = response.status().as_u16();
    let text = response.body_mut().read_to_string().unwrap_or_default();
    let value = serde_json::from_str(&text).unwrap_or(Value::Null);
    Ok((status, value))
}

fn api_error(status: u16, body: &Value) -> CatermError {
    match body.get("error").and_then(Value::as_str) {
        Some(code) => pro_err(code),
        None => pro_err(format!("HTTP_{status}")),
    }
}

/// Server ids (UUIDs) are interpolated into paths: refuse anything else.
fn path_id(id: &str) -> Result<&str, CatermError> {
    if !id.is_empty()
        && id.len() <= 64
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        Ok(id)
    } else {
        Err(pro_err("BAD_REQUEST"))
    }
}

fn parse<T: serde::de::DeserializeOwned>(body: &Value, key: &str) -> Option<T> {
    body.get(key)
        .filter(|v| !v.is_null())
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

// ---- Wire types ------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProAccount {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProLicense {
    pub status: String,
    pub tier: String,
    pub trial_ends_at: Option<i64>,
    pub current_period_end: Option<i64>,
    pub entitled: bool,
}

/// What the account may do right now: through its own licence (`tier` "pro") or as a member
/// of an entitled owner's team (`tier` "team").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProAccess {
    pub entitled: bool,
    pub tier: Option<String>,
    pub features: Vec<String>,
    pub device_limit: i64,
}

/// Another account as shown to someone else.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProPerson {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProMembership {
    pub team_id: String,
    pub owner: ProPerson,
    pub joined_at: i64,
    /// False while the owner is not subscribed.
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProTeamMember {
    pub account_id: String,
    pub email: String,
    pub name: String,
    pub joined_at: i64,
    pub active_devices: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProTeamInvite {
    pub id: String,
    pub email: String,
    pub invited_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProOwnedTeam {
    pub id: String,
    pub active: bool,
    pub members: Vec<ProTeamMember>,
    pub invites: Vec<ProTeamInvite>,
}

/// A pending invitation addressed to the signed-in account's email.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProInvitation {
    pub id: String,
    pub owner: ProPerson,
    pub invited_at: i64,
    pub expires_at: i64,
}

/// Everything team-related for the signed-in account; every team call returns it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProTeamView {
    pub owned: Option<ProOwnedTeam>,
    pub can_invite: bool,
    pub max_members: i64,
    pub membership: Option<ProMembership>,
    pub invitations: Vec<ProInvitation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProDevice {
    pub id: String,
    pub name: String,
    pub os: String,
    pub client_version: String,
    pub first_seen_at: i64,
    pub last_seen_at: i64,
    pub current: bool,
}

#[derive(Debug, Clone)]
struct Session {
    account: Value,
    refresh_token: String,
    access_token: String,
    access_expires_at: i64,
}

fn parse_session(body: &Value) -> Result<Session, CatermError> {
    let field = |k: &str| body.get(k).and_then(Value::as_str).map(str::to_string);
    Ok(Session {
        account: body
            .get("account")
            .cloned()
            .ok_or_else(|| pro_err("BAD_RESPONSE"))?,
        refresh_token: field("refresh_token").ok_or_else(|| pro_err("BAD_RESPONSE"))?,
        access_token: field("access_token").ok_or_else(|| pro_err("BAD_RESPONSE"))?,
        access_expires_at: body
            .get("access_expires_at")
            .and_then(Value::as_i64)
            .unwrap_or(0),
    })
}

fn account_from(v: &Value) -> Option<ProAccount> {
    serde_json::from_value(v.clone()).ok()
}

// ---- In-memory state -------------------------------------------------------------------------

/// Signed in on the lock screen while the vault was still locked; saved by `commit_pending`.
static PENDING: Lazy<Mutex<Option<Session>>> = Lazy::new(|| Mutex::new(None));
/// Short-lived access token for the persisted session.
static ACCESS: Lazy<Mutex<Option<(String, i64)>>> = Lazy::new(|| Mutex::new(None));

// ---- Local persistence (inside the encrypted vault database) ---------------------------------

fn ensure_table(conn: &Connection) -> Result<(), CatermError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS pro_state (
            id                    INTEGER PRIMARY KEY CHECK (id = 1),
            account_json          TEXT NOT NULL,
            refresh_token         TEXT NOT NULL,
            license_json          TEXT,
            token_payload         TEXT,
            token_signature       TEXT,
            token_key_version     INTEGER,
            last_seen_wall_clock  INTEGER NOT NULL DEFAULT 0,
            last_sync_at          INTEGER
        )",
    )
    .map_err(db_err)
}

struct StoredState {
    account: Value,
    refresh_token: String,
    license: Option<Value>,
    token: Option<(String, String, u32)>,
    last_seen: i64,
    last_sync_at: Option<i64>,
}

fn load_state(conn: &Connection) -> Result<Option<StoredState>, CatermError> {
    ensure_table(conn)?;
    conn.query_row(
        "SELECT account_json, refresh_token, license_json, token_payload, token_signature,
                token_key_version, last_seen_wall_clock, last_sync_at
         FROM pro_state WHERE id = 1",
        [],
        |r| {
            let account: String = r.get(0)?;
            let license: Option<String> = r.get(2)?;
            let payload: Option<String> = r.get(3)?;
            let signature: Option<String> = r.get(4)?;
            let version: Option<u32> = r.get(5)?;
            Ok(StoredState {
                account: serde_json::from_str(&account).unwrap_or(Value::Null),
                refresh_token: r.get(1)?,
                license: license.and_then(|l| serde_json::from_str(&l).ok()),
                token: match (payload, signature, version) {
                    (Some(p), Some(s), Some(v)) => Some((p, s, v)),
                    _ => None,
                },
                last_seen: r.get(6)?,
                last_sync_at: r.get(7)?,
            })
        },
    )
    .optional()
    .map_err(db_err)
}

fn save_session(conn: &Connection, session: &Session) -> Result<(), CatermError> {
    ensure_table(conn)?;
    let previous_account = load_state(conn)?
        .and_then(|s| account_from(&s.account))
        .map(|a| a.id);
    let same_account =
        previous_account.as_deref() == account_from(&session.account).map(|a| a.id).as_deref();
    let account_json = session.account.to_string();
    if same_account {
        conn.execute(
            "UPDATE pro_state SET account_json = ?1, refresh_token = ?2 WHERE id = 1",
            params![account_json, session.refresh_token],
        )
        .map_err(db_err)?;
    } else {
        // Different (or first) account: the previous account's licence must not carry over.
        conn.execute(
            "INSERT OR REPLACE INTO pro_state (id, account_json, refresh_token) VALUES (1, ?1, ?2)",
            params![account_json, session.refresh_token],
        )
        .map_err(db_err)?;
    }
    *ACCESS.lock() = Some((session.access_token.clone(), session.access_expires_at));
    Ok(())
}

fn save_license(
    conn: &Connection,
    license: &Value,
    token: Option<&SignedToken>,
) -> Result<(), CatermError> {
    let license_json = if license.is_null() {
        None
    } else {
        Some(license.to_string())
    };
    conn.execute(
        "UPDATE pro_state SET license_json = ?1, token_payload = ?2, token_signature = ?3,
                              token_key_version = ?4, last_sync_at = ?5
         WHERE id = 1",
        params![
            license_json,
            token.map(|t| t.payload.clone()),
            token.map(|t| t.signature.clone()),
            token.map(|t| t.key_version),
            now()
        ],
    )
    .map_err(db_err)?;
    Ok(())
}

fn clear_local(conn: &Connection) -> Result<(), CatermError> {
    ensure_table(conn)?;
    conn.execute("DELETE FROM pro_state", []).map_err(db_err)?;
    *ACCESS.lock() = None;
    Ok(())
}

fn open_db() -> Result<Connection, CatermError> {
    crate::db::open()
}

// ---- Access tokens ---------------------------------------------------------------------------

/// Returns a valid access token, rotating the refresh token when needed. A refused refresh
/// means the session was revoked (password reset, logout elsewhere): sign out locally.
fn access_token(conn: &Connection) -> Result<String, CatermError> {
    if let Some((token, exp)) = ACCESS.lock().clone()
        && exp > now() + 30
    {
        return Ok(token);
    }
    let state = load_state(conn)?.ok_or_else(|| pro_err("NOT_SIGNED_IN"))?;
    let (status, body) = request(
        "POST",
        "/auth/refresh",
        None,
        Some(&json!({ "refresh_token": state.refresh_token, "hwid": hwid()? })),
    )?;
    if status == 401 {
        clear_local(conn)?;
        return Err(pro_err("SESSION_EXPIRED"));
    }
    if status != 200 {
        return Err(api_error(status, &body));
    }
    let session = parse_session(&body)?;
    save_session(conn, &session)?;
    Ok(session.access_token)
}

fn authed(
    conn: &Connection,
    method: &str,
    path: &str,
    body: Option<&Value>,
) -> Result<(u16, Value), CatermError> {
    let token = access_token(conn)?;
    let (status, value) = request(method, path, Some(&token), body)?;
    if status == 401 {
        // Access token rejected (e.g. server secret rotated): refresh once and retry.
        *ACCESS.lock() = None;
        let token = access_token(conn)?;
        return request(method, path, Some(&token), body);
    }
    Ok((status, value))
}

// ---- Public API (called from Tauri commands) -------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProStatus {
    pub signed_in: bool,
    /// Signed in on the lock screen; saved once the vault unlocks.
    pub pending: bool,
    pub account: Option<ProAccount>,
    pub license: Option<ProLicense>,
    pub entitlement: Entitlement,
    pub last_sync_at: Option<i64>,
    /// False in builds without an embedded licence public key (dev builds, forks).
    pub key_configured: bool,
}

/// This device could not be activated: the account already uses `limit` devices.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceLimit {
    pub limit: i64,
    pub devices: Vec<ProDevice>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOutcome {
    pub status: ProStatus,
    pub device_limit: Option<DeviceLimit>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDetails {
    pub account: Option<ProAccount>,
    pub license: Option<ProLicense>,
    pub access: Option<ProAccess>,
    pub membership: Option<ProMembership>,
    pub devices: Vec<ProDevice>,
}

/// # Infallible: a reachability probe — an unreachable or disabled server *is* the answer
/// (`false`), so there is no separate error to report.
pub fn server_available() -> bool {
    matches!(request("GET", "/health", None, None), Ok((200, body)) if body.get("enabled").and_then(Value::as_bool) == Some(true))
}

pub fn register(email: &str, password: &str, name: &str, locale: &str) -> Result<(), CatermError> {
    let (status, body) = request(
        "POST",
        "/auth/register",
        None,
        Some(
            &json!({ "email": email.trim(), "password": password, "name": name.trim(), "locale": locale }),
        ),
    )?;
    if status == 201 {
        Ok(())
    } else {
        Err(api_error(status, &body))
    }
}

pub fn resend_verification(email: &str) -> Result<(), CatermError> {
    request(
        "POST",
        "/auth/resend-verification",
        None,
        Some(&json!({ "email": email.trim() })),
    )
    .map(|_| ())
}

pub fn forgot_password(email: &str, locale: &str) -> Result<(), CatermError> {
    request(
        "POST",
        "/auth/password/forgot",
        None,
        Some(&json!({ "email": email.trim(), "locale": locale })),
    )
    .map(|_| ())
}

/// Signs in. With the vault unlocked the session is saved immediately; on the lock screen it
/// waits in memory for [`commit_pending`].
pub fn login(email: &str, password: &str) -> Result<ProAccount, CatermError> {
    let hwid = hwid()?;
    let body = json!({
        "email": email.trim(),
        "password": password,
        "hwid": hwid,
        "device_name": device_name(),
        "os": std::env::consts::OS,
        "client_version": CLIENT_VERSION,
    });
    let (status, value) = request("POST", "/auth/login", None, Some(&body))?;
    if status != 200 {
        return Err(api_error(status, &value));
    }
    let session = parse_session(&value)?;
    let account = account_from(&session.account).ok_or_else(|| pro_err("BAD_RESPONSE"))?;
    if crate::vault::is_unlocked().unwrap_or(false) {
        save_session(&open_db()?, &session)?;
    } else {
        *PENDING.lock() = Some(session);
    }
    Ok(account)
}

/// Saves a lock-screen sign-in into the now-unlocked vault. Returns false if there was none.
pub fn commit_pending() -> Result<bool, CatermError> {
    let Some(session) = PENDING.lock().take() else {
        return Ok(false);
    };
    save_session(&open_db()?, &session)?;
    Ok(true)
}

/// Offline status: what this device is entitled to right now, from the stored signed token.
pub fn status() -> Result<ProStatus, CatermError> {
    if !crate::vault::is_unlocked().unwrap_or(false) {
        let pending = PENDING
            .lock()
            .as_ref()
            .and_then(|s| account_from(&s.account));
        return Ok(ProStatus {
            signed_in: pending.is_some(),
            pending: pending.is_some(),
            account: pending,
            license: None,
            entitlement: Entitlement::None,
            last_sync_at: None,
            key_configured: key_configured(),
        });
    }
    let conn = open_db()?;
    let Some(state) = load_state(&conn)? else {
        return Ok(ProStatus {
            signed_in: false,
            pending: false,
            account: None,
            license: None,
            entitlement: Entitlement::None,
            last_sync_at: None,
            key_configured: key_configured(),
        });
    };
    let hwid = hwid()?;
    let token = state
        .token
        .as_ref()
        .map(|(p, s, v)| (p.as_str(), s.as_str(), *v));
    let (entitlement, seen) = evaluate(token, public_key, &hwid, state.last_seen, now());
    if seen != state.last_seen {
        conn.execute(
            "UPDATE pro_state SET last_seen_wall_clock = ?1 WHERE id = 1",
            params![seen],
        )
        .map_err(db_err)?;
    }
    Ok(ProStatus {
        signed_in: true,
        pending: false,
        account: account_from(&state.account),
        license: state
            .license
            .as_ref()
            .and_then(|l| serde_json::from_value(l.clone()).ok()),
        entitlement,
        last_sync_at: state.last_sync_at,
        key_configured: key_configured(),
    })
}

fn apply_license_response(
    conn: &Connection,
    status: u16,
    body: &Value,
) -> Result<SyncOutcome, CatermError> {
    if status == 409 && body.get("error").and_then(Value::as_str) == Some("DEVICE_LIMIT_EXCEEDED") {
        let limit = DeviceLimit {
            limit: body
                .get("device_limit")
                .and_then(Value::as_i64)
                .unwrap_or(1),
            devices: parse(body, "devices").unwrap_or_default(),
        };
        return Ok(SyncOutcome {
            status: self::status()?,
            device_limit: Some(limit),
        });
    }
    if !(200..300).contains(&status) {
        return Err(api_error(status, body));
    }
    let license = body.get("license").cloned().unwrap_or(Value::Null);
    let token: Option<SignedToken> = body
        .get("token")
        .filter(|t| !t.is_null())
        .and_then(|t| serde_json::from_value(t.clone()).ok());
    save_license(conn, &license, token.as_ref())?;
    Ok(SyncOutcome {
        status: self::status()?,
        device_limit: None,
    })
}

/// Heartbeat: activates this device (if a slot is free) and refreshes the offline token.
pub fn sync() -> Result<SyncOutcome, CatermError> {
    let conn = open_db()?;
    let body = device_body(&hwid()?);
    let (status, value) = authed(&conn, "POST", "/license/heartbeat", Some(&body))?;
    apply_license_response(&conn, status, &value)
}

pub fn start_trial() -> Result<SyncOutcome, CatermError> {
    let conn = open_db()?;
    let body = device_body(&hwid()?);
    let (status, value) = authed(&conn, "POST", "/license/trial", Some(&body))?;
    apply_license_response(&conn, status, &value)
}

pub fn account_details() -> Result<AccountDetails, CatermError> {
    let conn = open_db()?;
    let (status, body) = authed(&conn, "GET", &format!("/account?hwid={}", hwid()?), None)?;
    if status != 200 {
        return Err(api_error(status, &body));
    }
    Ok(AccountDetails {
        account: body.get("account").and_then(account_from),
        license: parse(&body, "license"),
        access: parse(&body, "access"),
        membership: parse(&body, "membership"),
        devices: parse(&body, "devices").unwrap_or_default(),
    })
}

pub fn revoke_device(device_id: &str) -> Result<(), CatermError> {
    let conn = open_db()?;
    let path = format!("/license/devices/{}/revoke", path_id(device_id)?);
    let (status, body) = authed(&conn, "POST", &path, None)?;
    if status == 204 || status == 200 {
        Ok(())
    } else {
        Err(api_error(status, &body))
    }
}

// ---- Hosted AI (OmniRoute, via GCC's pooled proxy) --------------------------------------------

/// The signed-in account's hosted-AI pool for the current period. `entitled` mirrors the
/// `ai_hosted` feature; `pooled` is true when the quota belongs to a team owner rather than
/// this account's own license.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ProAiUsage {
    pub enabled: bool,
    pub entitled: bool,
    pub pooled: bool,
    pub period: String,
    pub used: i64,
    pub limit: i64,
    pub remaining: i64,
    pub resets_at: i64,
}

pub fn ai_usage() -> Result<ProAiUsage, CatermError> {
    let conn = open_db()?;
    let (status, body) = authed(&conn, "GET", "/ai/usage", None)?;
    if status != 200 {
        return Err(api_error(status, &body));
    }
    serde_json::from_value(body).map_err(|_| pro_err("BAD_RESPONSE"))
}

/// Sends an OpenAI-compatible chat payload through GCC's `/ai/chat/completions`, which counts
/// the request against the pooled quota and forwards it to OmniRoute with the server's own
/// key — this device never sees the upstream URL or key. `payload` should carry `messages` and
/// may omit `model`; the server assigns the pooled model. Returns the raw OpenAI-shaped
/// response body (not the extracted text), so callers can reuse existing parsers.
pub(crate) fn ai_chat_completion(payload: &Value) -> Result<Value, CatermError> {
    let conn = open_db()?;
    let (status, body) = authed(&conn, "POST", "/ai/chat/completions", Some(payload))?;
    if status != 200 {
        return Err(api_error(status, &body));
    }
    Ok(body)
}

// ---- Team ------------------------------------------------------------------------------------

fn team_call(method: &str, path: &str, body: Option<&Value>) -> Result<ProTeamView, CatermError> {
    let conn = open_db()?;
    let (status, value) = authed(&conn, method, path, body)?;
    if !(200..300).contains(&status) {
        return Err(api_error(status, &value));
    }
    serde_json::from_value(value).map_err(|_| pro_err("BAD_RESPONSE"))
}

pub fn team() -> Result<ProTeamView, CatermError> {
    team_call("GET", "/team", None)
}

/// Invites `email` to the signed-in owner's team; `locale` is used for the invite email when
/// the invitee has no account yet.
pub fn team_invite(email: &str, locale: &str) -> Result<ProTeamView, CatermError> {
    team_call(
        "POST",
        "/team/invites",
        Some(&json!({ "email": email.trim(), "locale": locale })),
    )
}

pub fn team_cancel_invite(invite_id: &str) -> Result<ProTeamView, CatermError> {
    team_call(
        "DELETE",
        &format!("/team/invites/{}", path_id(invite_id)?),
        None,
    )
}

pub fn team_remove_member(account_id: &str) -> Result<ProTeamView, CatermError> {
    team_call(
        "DELETE",
        &format!("/team/members/{}", path_id(account_id)?),
        None,
    )
}

/// Joining or leaving changes what this device may do: callers should [`sync`] afterwards.
pub fn team_accept(invitation_id: &str) -> Result<ProTeamView, CatermError> {
    team_call(
        "POST",
        &format!("/team/invitations/{}/accept", path_id(invitation_id)?),
        None,
    )
}

pub fn team_decline(invitation_id: &str) -> Result<ProTeamView, CatermError> {
    team_call(
        "POST",
        &format!("/team/invitations/{}/decline", path_id(invitation_id)?),
        None,
    )
}

pub fn team_leave() -> Result<ProTeamView, CatermError> {
    team_call("POST", "/team/leave", None)
}

/// Signs out on this device: revokes the refresh token on the server (best effort — being
/// offline must not keep someone signed in) and forgets the local session and licence.
pub fn logout() -> Result<(), CatermError> {
    *PENDING.lock() = None;
    let conn = open_db()?;
    if let Some(state) = load_state(&conn)? {
        let _ = request(
            "POST",
            "/auth/logout",
            None,
            Some(&json!({ "refresh_token": state.refresh_token })),
        );
    }
    clear_local(&conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn token(hwid: &str, exp: i64) -> (String, String) {
        let payload = json!({ "lic_id": "l", "account_id": "a", "hwid": hwid, "tier": "pro",
            "features": ["cloud_sync"], "iat": 0, "exp": exp, "nonce": "n" })
        .to_string();
        let sig = STANDARD.encode(key().sign(payload.as_bytes()).to_bytes());
        (payload, sig)
    }

    fn eval(payload: &str, sig: &str, hwid: &str, last_seen: i64, now: i64) -> (Entitlement, i64) {
        let vk = key().verifying_key();
        evaluate(
            Some((payload, sig, 1)),
            |v| (v == 1).then_some(vk),
            hwid,
            last_seen,
            now,
        )
    }

    #[test]
    fn valid_token_is_valid_and_ratchets_last_seen() {
        let (p, s) = token("dev", 2_000);
        let (ent, seen) = eval(&p, &s, "dev", 900, 1_000);
        assert!(matches!(
            ent,
            Entitlement::Valid {
                expires_at: 2_000,
                ..
            }
        ));
        assert_eq!(seen, 1_000);
    }

    #[test]
    fn expired_token_is_expired() {
        let (p, s) = token("dev", 2_000);
        assert_eq!(eval(&p, &s, "dev", 0, 2_000).0, Entitlement::Expired);
    }

    #[test]
    fn tampered_payload_is_invalid() {
        let (p, s) = token("dev", 2_000);
        let forged = p.replace("2000", "9999999999");
        assert_eq!(
            eval(&forged, &s, "dev", 0, 1_000).0,
            Entitlement::Invalid {
                reason: "BAD_SIGNATURE".into()
            }
        );
    }

    #[test]
    fn token_from_another_machine_is_invalid() {
        let (p, s) = token("other", 2_000);
        assert_eq!(
            eval(&p, &s, "dev", 0, 1_000).0,
            Entitlement::Invalid {
                reason: "OTHER_DEVICE".into()
            }
        );
    }

    #[test]
    fn clock_rolled_back_requires_revalidation() {
        let (p, s) = token("dev", 10_000);
        // Last seen at 5000, now claims 1000: far beyond NTP drift.
        let (ent, seen) = eval(&p, &s, "dev", 5_000, 1_000);
        assert_eq!(
            ent,
            Entitlement::RevalidationRequired {
                reason: "CLOCK_ROLLBACK".into()
            }
        );
        assert_eq!(seen, 5_000, "last_seen never moves backwards");
        // Small NTP corrections are tolerated.
        assert!(matches!(
            eval(&p, &s, "dev", 5_000, 4_900).0,
            Entitlement::Valid { .. }
        ));
    }

    #[test]
    fn unknown_key_version_is_invalid() {
        let (p, s) = token("dev", 2_000);
        let (ent, _) = evaluate(Some((&p, &s, 9)), |_| None, "dev", 0, 1_000);
        assert_eq!(
            ent,
            Entitlement::Invalid {
                reason: "KEY_NOT_CONFIGURED".into()
            }
        );
    }

    #[test]
    fn no_token_means_none() {
        assert_eq!(evaluate(None, |_| None, "dev", 0, 0).0, Entitlement::None);
    }

    #[test]
    fn server_error_codes_become_pro_errors() {
        let err = api_error(401, &json!({ "error": "INVALID_CREDENTIALS" }));
        assert_eq!(err.code(), "CAT-PRO-000");
        assert_eq!(err.to_string(), "pro: INVALID_CREDENTIALS");
        assert_eq!(api_error(502, &Value::Null).to_string(), "pro: HTTP_502");
    }

    #[test]
    fn member_token_carries_team_tier() {
        let payload = json!({ "lic_id": "owner-lic", "account_id": "m", "hwid": "dev", "tier": "team",
            "features": ["ai_hosted", "team_space"], "team_id": "t1", "iat": 0, "exp": 2_000, "nonce": "n" })
        .to_string();
        let sig = STANDARD.encode(key().sign(payload.as_bytes()).to_bytes());
        match eval(&payload, &sig, "dev", 0, 1_000).0 {
            Entitlement::Valid { tier, features, .. } => {
                assert_eq!(tier, "team");
                assert_eq!(features, vec!["ai_hosted", "team_space"]);
            }
            other => panic!("expected valid, got {other:?}"),
        }
    }

    #[test]
    fn team_view_parses_server_shape() {
        let body = json!({
            "owned": { "id": "t", "active": true,
                "members": [{ "account_id": "a", "email": "m@x.io", "name": "M", "joined_at": 1, "active_devices": 1 }],
                "invites": [{ "id": "i", "email": "n@x.io", "invited_at": 2, "expires_at": 3 }] },
            "can_invite": true, "max_members": 6, "membership": null,
            "invitations": [{ "id": "j", "owner": { "email": "o@x.io", "name": "" }, "invited_at": 4, "expires_at": 5 }]
        });
        let view: ProTeamView = serde_json::from_value(body).unwrap();
        assert_eq!(view.owned.as_ref().unwrap().members[0].active_devices, 1);
        assert_eq!(view.invitations[0].owner.email, "o@x.io");
        let out = serde_json::to_value(&view).unwrap();
        assert_eq!(out["maxMembers"], 6);
        assert_eq!(out["owned"]["members"][0]["activeDevices"], 1);
    }

    #[test]
    fn path_ids_are_restricted() {
        assert!(path_id("0b8f5a8e-1c2d-4e5f-8a9b-0c1d2e3f4a5b").is_ok());
        for bad in ["", "../account", "a/b", "a?b=c", "a b"] {
            assert!(path_id(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn hwid_is_stable_hex() {
        let a = hwid().unwrap();
        assert_eq!(a, hwid().unwrap());
        assert_eq!(a.len(), 64);
    }
}
