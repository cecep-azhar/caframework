use caf_core::billing::client::{BillingConfig, GccBillingClient};
use caf_core::billing::state::EffectiveTier;
use caf_core::billing::store;
use caf_xtask::gcc_mock::{create_mock_app, init_dev_keys, MockState};
use base64::Engine;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_billing_client_integration_against_mock() {
    let (signing_key, verifying_key, kid) = init_dev_keys();
    let pub_key_b64 = base64::engine::general_purpose::STANDARD.encode(verifying_key.as_bytes());

    let state = MockState {
        signing_key: Arc::new(signing_key),
        verifying_key,
        kid: kid.clone(),
        accounts: Arc::new(Mutex::new(HashMap::new())),
        devices: Arc::new(Mutex::new(HashMap::new())),
        checkouts: Arc::new(Mutex::new(HashMap::new())),
        license_keys: Arc::new(Mutex::new(HashMap::new())),
        scenario_bad_signature: Arc::new(Mutex::new(false)),
        scenario_error_500: Arc::new(Mutex::new(false)),
    };

    let app = create_mock_app(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let config = BillingConfig {
        enabled: true,
        product_code: "CACASH".into(),
        gcc_base_url: format!("http://127.0.0.1:{}", port),
        public_keys: vec![(kid, pub_key_b64)],
        offline_days: 7,
        refresh_hours: 6,
    };

    let client = GccBillingClient::new(config);

    // 1. Request OTP
    client.request_otp("user@example.com").expect("OTP request failed");

    // 2. Verify OTP & register device
    let auth = client
        .verify_otp("user@example.com", "123456", "dev-client-1", "MacBook Pro")
        .expect("OTP verify failed");
    assert_eq!(auth.account.email, "user@example.com");

    // Initial state -> Free
    let (payload, effective) = client.refresh_entitlement().expect("refresh failed");
    assert_eq!(payload.tier, "free");
    assert_eq!(effective.tier, EffectiveTier::Free);
    assert!(!effective.is_pro);

    // 3. Admin simulates upgrading account to Pro Monthly
    {
        let mut accounts = state.accounts.lock().unwrap();
        let acc = accounts.get_mut("user@example.com").unwrap();
        acc.tier = "pro".to_string();
        acc.billing_type = "monthly".to_string();
        acc.expires_at = Some("2030-01-01T00:00:00Z".to_string());
    }

    // Refresh -> Pro
    let (payload_pro, effective_pro) = client.refresh_entitlement().expect("refresh failed");
    assert_eq!(payload_pro.tier, "pro");
    assert_eq!(effective_pro.tier, EffectiveTier::Pro);
    assert!(effective_pro.is_pro);

    // 4. Test Checkout Flow (IDR)
    let plans = client.get_plans(Some("IDR")).expect("get plans failed");
    assert!(!plans.is_empty());

    let chk = client.create_checkout(&plans[0].plan_id).expect("create checkout failed");
    assert_eq!(chk.completion_mode, "poll");

    let status = client.poll_checkout_status(&chk.checkout_id).expect("poll status failed");
    assert_eq!(status, "pending");

    // 5. Test License Key Activation (Ko-fi flow)
    {
        let mut lic = state.license_keys.lock().unwrap();
        lic.insert(
            "KEY-KOFI-123456".to_string(),
            caf_xtask::gcc_mock::MockLicense {
                key: "KEY-KOFI-123456".to_string(),
                product_code: "CACASH".to_string(),
                tier: "pro".to_string(),
                billing_type: "lifetime".to_string(),
                revoked: false,
            },
        );
    }

    let auth_lic = client
        .activate_license("KEY-KOFI-123456", "dev-client-2", "Windows PC")
        .expect("activate license failed");
    assert_eq!(auth_lic.account.name, "License Holder");

    let (lic_payload, lic_effective) = client.refresh_entitlement().expect("refresh lic failed");
    assert_eq!(lic_payload.billing_type, "lifetime");
    assert_eq!(lic_effective.tier, EffectiveTier::Pro);
    assert!(lic_effective.is_pro);

    // 6. Test Devices List and Deactivate
    let devices = client.list_devices().expect("list devices failed");
    assert!(!devices.is_empty());

    client.deactivate_device("dev-client-2").expect("deactivate failed");
    assert!(store::get_device_token().unwrap().is_none());
}
