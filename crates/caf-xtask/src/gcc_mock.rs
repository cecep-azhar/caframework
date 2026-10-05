//! Local Mock GCC Billing Hub Server (`cargo xtask gcc-mock`).

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post},
};
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockState {
    pub signing_key: Arc<SigningKey>,
    pub verifying_key: VerifyingKey,
    pub kid: String,
    pub accounts: Arc<Mutex<HashMap<String, MockAccount>>>, // email -> MockAccount
    pub devices: Arc<Mutex<HashMap<String, MockDevice>>>,   // device_token -> MockDevice
    pub checkouts: Arc<Mutex<HashMap<String, MockCheckout>>>, // checkout_id -> MockCheckout
    pub license_keys: Arc<Mutex<HashMap<String, MockLicense>>>, // key -> MockLicense
    pub scenario_bad_signature: Arc<Mutex<bool>>,
    pub scenario_error_500: Arc<Mutex<bool>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MockAccount {
    pub id: String,
    pub email: String,
    pub name: String,
    pub tier: String,
    pub billing_type: String,
    pub status: String,
    pub expires_at: Option<String>,
    pub grace_until: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MockDevice {
    pub id: String,
    pub account_id: String,
    pub product_code: String,
    pub device_id: String,
    pub device_name: String,
    pub device_token: String,
    pub revoked: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MockCheckout {
    pub id: String,
    pub account_id: String,
    pub plan_id: String,
    pub status: String,
    pub completion_mode: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MockLicense {
    pub key: String,
    pub product_code: String,
    pub tier: String,
    pub billing_type: String,
    pub revoked: bool,
}

pub fn create_mock_app(state: MockState) -> Router {
    Router::new()
        // Client API per OpenAPI contract
        .route("/v1/auth/otp/request", post(handle_otp_request))
        .route("/v1/auth/otp/verify", post(handle_otp_verify))
        .route("/v1/licenses/activate", post(handle_license_activate))
        .route("/v1/entitlement", get(handle_get_entitlement))
        .route("/v1/plans", get(handle_get_plans))
        .route("/v1/checkout", post(handle_create_checkout))
        .route("/v1/checkout/{id}", get(handle_get_checkout_status))
        .route("/v1/devices", get(handle_list_devices))
        .route("/v1/devices/{id}", delete(handle_delete_device))
        .route("/.well-known/gcc-billing-keys", get(handle_public_keys))
        // Admin endpoints for testing
        .route("/admin/set-plan", post(admin_set_plan))
        .route("/admin/issue-license", post(admin_issue_license))
        .route("/admin/simulate-paid/{id}", post(admin_simulate_paid))
        .route("/admin/scenario", post(admin_set_scenario))
        .with_state(state)
}

pub fn init_dev_keys() -> (SigningKey, VerifyingKey, String) {
    let mut key_bytes = [0u8; 32];
    use rand::RngCore;
    OsRng.fill_bytes(&mut key_bytes);
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = signing_key.verifying_key();
    let kid = "gcc-2026-10".to_string();
    (signing_key, verifying_key, kid)
}

// --------------------------------------------------------------------------
// Request / Response structures
// --------------------------------------------------------------------------

#[derive(Deserialize)]
struct OtpRequestPayload {
    email: String,
    product_code: String,
}

#[derive(Deserialize)]
struct OtpVerifyPayload {
    email: String,
    otp_code: String,
    product_code: String,
    device_id: String,
    device_name: String,
}

#[derive(Deserialize)]
struct LicenseActivatePayload {
    license_key: String,
    product_code: String,
    device_id: String,
    device_name: String,
}

#[derive(Deserialize)]
struct PlansQuery {
    product_code: String,
    currency: Option<String>,
}

#[derive(Deserialize)]
struct CheckoutPayload {
    plan_id: String,
}

// --------------------------------------------------------------------------
// Handlers
// --------------------------------------------------------------------------

async fn handle_otp_request(
    State(_state): State<MockState>,
    Json(payload): Json<OtpRequestPayload>,
) -> impl IntoResponse {
    println!("[GCC-MOCK] OTP for email {}: 123456", payload.email);
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "message": "OTP sent" })),
    )
}

async fn handle_otp_verify(
    State(state): State<MockState>,
    Json(payload): Json<OtpVerifyPayload>,
) -> impl IntoResponse {
    if payload.otp_code != "123456" {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid OTP" })),
        );
    }

    let mut accounts = state.accounts.lock().unwrap();
    let account = accounts
        .entry(payload.email.clone())
        .or_insert_with(|| MockAccount {
            id: uuid::Uuid::new_v4().to_string(),
            email: payload.email.clone(),
            name: payload
                .email
                .split('@')
                .next()
                .unwrap_or("User")
                .to_string(),
            tier: "free".to_string(),
            billing_type: "free".to_string(),
            status: "active".to_string(),
            expires_at: None,
            grace_until: None,
        });

    let device_token = format!("tok_{}", uuid::Uuid::new_v4());
    let mut devices = state.devices.lock().unwrap();
    devices.insert(
        device_token.clone(),
        MockDevice {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account.id.clone(),
            product_code: payload.product_code,
            device_id: payload.device_id,
            device_name: payload.device_name,
            device_token: device_token.clone(),
            revoked: false,
        },
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "device_token": device_token,
            "account": {
                "id": account.id,
                "email": account.email,
                "name": account.name
            }
        })),
    )
}

async fn handle_license_activate(
    State(state): State<MockState>,
    Json(payload): Json<LicenseActivatePayload>,
) -> impl IntoResponse {
    let licenses = state.license_keys.lock().unwrap();
    let lic = match licenses.get(&payload.license_key) {
        Some(l) => l,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Unknown license key" })),
            );
        }
    };

    if lic.revoked {
        return (
            StatusCode::GONE,
            Json(serde_json::json!({ "error": "License revoked" })),
        );
    }

    let email = format!("license_{}@example.com", &payload.license_key[..6]);
    let mut accounts = state.accounts.lock().unwrap();
    let account = accounts
        .entry(email.clone())
        .or_insert_with(|| MockAccount {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.clone(),
            name: "License Holder".to_string(),
            tier: lic.tier.clone(),
            billing_type: lic.billing_type.clone(),
            status: "active".to_string(),
            expires_at: None,
            grace_until: None,
        });
    account.tier = lic.tier.clone();
    account.billing_type = lic.billing_type.clone();

    let device_token = format!("tok_{}", uuid::Uuid::new_v4());
    let mut devices = state.devices.lock().unwrap();
    devices.insert(
        device_token.clone(),
        MockDevice {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account.id.clone(),
            product_code: payload.product_code,
            device_id: payload.device_id,
            device_name: payload.device_name,
            device_token: device_token.clone(),
            revoked: false,
        },
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "device_token": device_token,
            "account": {
                "id": account.id,
                "email": account.email,
                "name": account.name
            }
        })),
    )
}

async fn handle_get_entitlement(
    State(state): State<MockState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if *state.scenario_error_500.lock().unwrap() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Server error scenario" })),
        );
    }

    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(a) if a.starts_with("Bearer ") => &a[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing token" })),
            );
        }
    };

    let devices = state.devices.lock().unwrap();
    let device = match devices.get(auth_header) {
        Some(d) if !d.revoked => d,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Device revoked or not found" })),
            );
        }
    };

    let accounts = state.accounts.lock().unwrap();
    let account = accounts
        .values()
        .find(|a| a.id == device.account_id)
        .cloned()
        .unwrap_or(MockAccount {
            id: device.account_id.clone(),
            email: "unknown@example.com".to_string(),
            name: "User".to_string(),
            tier: "free".to_string(),
            billing_type: "free".to_string(),
            status: "active".to_string(),
            expires_at: None,
            grace_until: None,
        });

    let product_code = params
        .get("product_code")
        .cloned()
        .unwrap_or(device.product_code.clone());

    let payload = serde_json::json!({
        "v": 1,
        "kid": state.kid,
        "account_id": account.id,
        "product_code": product_code,
        "device_id": device.device_id,
        "tier": account.tier,
        "billing_type": account.billing_type,
        "status": account.status,
        "expires_at": account.expires_at,
        "grace_until": account.grace_until,
        "issued_at": chrono::Utc::now().to_rfc3339()
    });

    // Create compact JWS: header.payload.signature
    let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&serde_json::json!({"alg": "EdDSA", "typ": "JWT", "kid": state.kid}))
            .unwrap(),
    );
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&payload).unwrap());
    let signing_input = format!("{}.{}", header_b64, payload_b64);

    let signature_bytes = if *state.scenario_bad_signature.lock().unwrap() {
        [0u8; 64]
    } else {
        state.signing_key.sign(signing_input.as_bytes()).to_bytes()
    };
    let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(signature_bytes);
    let token = format!("{}.{}", signing_input, sig_b64);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "token": token,
            "payload": payload
        })),
    )
}

async fn handle_get_plans(
    State(_state): State<MockState>,
    Query(query): Query<PlansQuery>,
) -> impl IntoResponse {
    let currency = query.currency.unwrap_or_else(|| "IDR".to_string());
    let plans = if currency == "USD" {
        vec![
            serde_json::json!({
                "plan_id": "pro_monthly_usd",
                "billing_type": "monthly",
                "price_minor": 900,
                "currency": "USD"
            }),
            serde_json::json!({
                "plan_id": "pro_yearly_usd",
                "billing_type": "yearly",
                "price_minor": 8900,
                "currency": "USD"
            }),
            serde_json::json!({
                "plan_id": "pro_lifetime_usd",
                "billing_type": "lifetime",
                "price_minor": 19900,
                "currency": "USD"
            }),
        ]
    } else {
        vec![
            serde_json::json!({
                "plan_id": "pro_monthly_idr",
                "billing_type": "monthly",
                "price_minor": 149000,
                "currency": "IDR"
            }),
            serde_json::json!({
                "plan_id": "pro_yearly_idr",
                "billing_type": "yearly",
                "price_minor": 1490000,
                "currency": "IDR"
            }),
            serde_json::json!({
                "plan_id": "pro_lifetime_idr",
                "billing_type": "lifetime",
                "price_minor": 2990000,
                "currency": "IDR"
            }),
        ]
    };

    (StatusCode::OK, Json(plans))
}

async fn handle_create_checkout(
    State(state): State<MockState>,
    headers: HeaderMap,
    Json(payload): Json<CheckoutPayload>,
) -> impl IntoResponse {
    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(a) if a.starts_with("Bearer ") => &a[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Missing token" })),
            );
        }
    };

    let devices = state.devices.lock().unwrap();
    let device = match devices.get(auth_header) {
        Some(d) => d,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Unauthorized" })),
            );
        }
    };

    let checkout_id = format!("chk_{}", uuid::Uuid::new_v4());
    let is_usd = payload.plan_id.contains("usd");
    let completion_mode = if is_usd { "license_key" } else { "poll" };
    let checkout_url = if is_usd {
        "https://ko-fi.com/fathforce".to_string()
    } else {
        format!("https://pub.mayar.id/pay/{}", checkout_id)
    };

    let mut checkouts = state.checkouts.lock().unwrap();
    checkouts.insert(
        checkout_id.clone(),
        MockCheckout {
            id: checkout_id.clone(),
            account_id: device.account_id.clone(),
            plan_id: payload.plan_id,
            status: "pending".to_string(),
            completion_mode: completion_mode.to_string(),
        },
    );

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "checkout_id": checkout_id,
            "checkout_url": checkout_url,
            "completion_mode": completion_mode
        })),
    )
}

async fn handle_get_checkout_status(
    State(state): State<MockState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let checkouts = state.checkouts.lock().unwrap();
    if let Some(chk) = checkouts.get(&id) {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "status": chk.status })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Checkout not found" })),
        )
    }
}

async fn handle_list_devices(
    State(state): State<MockState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(a) if a.starts_with("Bearer ") => &a[7..],
        _ => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!([]))),
    };

    let devices = state.devices.lock().unwrap();
    let curr_device = match devices.get(auth_header) {
        Some(d) => d,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!([]))),
    };

    let list: Vec<_> = devices
        .values()
        .filter(|d| d.account_id == curr_device.account_id && !d.revoked)
        .map(|d| {
            serde_json::json!({
                "id": d.id,
                "device_id": d.device_id,
                "device_name": d.device_name,
                "is_current": d.device_token == curr_device.device_token,
                "last_seen_at": chrono::Utc::now().to_rfc3339()
            })
        })
        .collect();

    (StatusCode::OK, Json(serde_json::json!(list)))
}

async fn handle_delete_device(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let auth_header = match headers.get("Authorization").and_then(|h| h.to_str().ok()) {
        Some(a) if a.starts_with("Bearer ") => &a[7..],
        _ => return StatusCode::UNAUTHORIZED,
    };

    let mut devices = state.devices.lock().unwrap();
    if let Some(dev) = devices
        .values_mut()
        .find(|d| d.id == id || d.device_token == auth_header)
    {
        dev.revoked = true;
    }

    StatusCode::NO_CONTENT
}

async fn handle_public_keys(State(state): State<MockState>) -> impl IntoResponse {
    let pub_key_b64 =
        base64::engine::general_purpose::STANDARD.encode(state.verifying_key.as_bytes());
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "keys": [
                {
                    "kid": state.kid,
                    "ed25519": pub_key_b64
                }
            ]
        })),
    )
}

// --------------------------------------------------------------------------
// Admin Endpoints for Simulation & Tests
// --------------------------------------------------------------------------

#[derive(Deserialize)]
struct AdminSetPlanPayload {
    email: String,
    tier: String,
    billing_type: String,
    status: Option<String>,
    expires_at: Option<String>,
    grace_until: Option<String>,
}

async fn admin_set_plan(
    State(state): State<MockState>,
    Json(payload): Json<AdminSetPlanPayload>,
) -> impl IntoResponse {
    let mut accounts = state.accounts.lock().unwrap();
    if let Some(acc) = accounts.get_mut(&payload.email) {
        acc.tier = payload.tier;
        acc.billing_type = payload.billing_type;
        if let Some(st) = payload.status {
            acc.status = st;
        }
        acc.expires_at = payload.expires_at;
        acc.grace_until = payload.grace_until;
    }
    StatusCode::OK
}

#[derive(Deserialize)]
struct AdminIssueLicensePayload {
    key: String,
    product_code: String,
    tier: String,
    billing_type: String,
}

async fn admin_issue_license(
    State(state): State<MockState>,
    Json(payload): Json<AdminIssueLicensePayload>,
) -> impl IntoResponse {
    let mut lic = state.license_keys.lock().unwrap();
    lic.insert(
        payload.key.clone(),
        MockLicense {
            key: payload.key,
            product_code: payload.product_code,
            tier: payload.tier,
            billing_type: payload.billing_type,
            revoked: false,
        },
    );
    StatusCode::CREATED
}

async fn admin_simulate_paid(
    State(state): State<MockState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut checkouts = state.checkouts.lock().unwrap();
    if let Some(chk) = checkouts.get_mut(&id) {
        chk.status = "paid".to_string();
        let mut accounts = state.accounts.lock().unwrap();
        if let Some(acc) = accounts.values_mut().find(|a| a.id == chk.account_id) {
            acc.tier = "pro".to_string();
            acc.billing_type = if chk.plan_id.contains("yearly") {
                "yearly".to_string()
            } else if chk.plan_id.contains("lifetime") {
                "lifetime".to_string()
            } else {
                "monthly".to_string()
            };
            acc.status = "active".to_string();
        }
    }
    StatusCode::OK
}

#[derive(Deserialize)]
struct AdminScenarioPayload {
    bad_signature: Option<bool>,
    error_500: Option<bool>,
}

async fn admin_set_scenario(
    State(state): State<MockState>,
    Json(payload): Json<AdminScenarioPayload>,
) -> impl IntoResponse {
    if let Some(bs) = payload.bad_signature {
        *state.scenario_bad_signature.lock().unwrap() = bs;
    }
    if let Some(e) = payload.error_500 {
        *state.scenario_error_500.lock().unwrap() = e;
    }
    StatusCode::OK
}

pub async fn run_mock_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let (signing_key, verifying_key, kid) = init_dev_keys();
    let state = MockState {
        signing_key: Arc::new(signing_key),
        verifying_key,
        kid,
        accounts: Arc::new(Mutex::new(HashMap::new())),
        devices: Arc::new(Mutex::new(HashMap::new())),
        checkouts: Arc::new(Mutex::new(HashMap::new())),
        license_keys: Arc::new(Mutex::new(HashMap::new())),
        scenario_bad_signature: Arc::new(Mutex::new(false)),
        scenario_error_500: Arc::new(Mutex::new(false)),
    };

    let app = create_mock_app(state.clone());
    let pub_key_b64 =
        base64::engine::general_purpose::STANDARD.encode(state.verifying_key.as_bytes());

    println!("============================================================");
    println!(
        "  GCC Billing Hub Mock Server running on http://127.0.0.1:{}",
        port
    );
    println!("  Public Key (kid: {}): {}", state.kid, pub_key_b64);
    println!("============================================================");

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
