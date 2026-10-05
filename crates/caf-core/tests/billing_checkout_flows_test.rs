use caf_core::billing::client::{BillingConfig, GccBillingClient};
use caf_core::billing::state::EffectiveTier;
use caf_core::billing::store;
use caf_xtask::gcc_mock::{create_mock_app, init_dev_keys, MockLicense, MockState};
use base64::Engine;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_checkout_modes_poll_and_license_key() {
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
        product_code: "CATERM".into(),
        gcc_base_url: format!("http://127.0.0.1:{}", port),
        public_keys: vec![(kid, pub_key_b64)],
        offline_days: 7,
        refresh_hours: 6,
    };

    let client = GccBillingClient::new(config);

    // Register initial device as Free
    let _auth = client
        .verify_otp("buyer@fathforce.com", "123456", "dev-buyer-1", "Linux Desktop")
        .expect("OTP verify failed");

    let (_, initial_state) = client.refresh_entitlement().expect("initial refresh failed");
    assert_eq!(initial_state.tier, EffectiveTier::Free);
    assert!(!initial_state.is_pro);

    // =========================================================================
    // Scenario 1: IDR Plan -> Completion Mode: "poll" (Mayar)
    // =========================================================================
    let idr_plans = client.get_plans(Some("IDR")).expect("get IDR plans failed");
    let monthly_plan = idr_plans.iter().find(|p| p.billing_type == "monthly").unwrap();
    
    let checkout = client.create_checkout(&monthly_plan.plan_id).expect("create checkout failed");
    assert_eq!(checkout.completion_mode, "poll");
    assert!(checkout.checkout_url.contains("pub.mayar.id"));

    // Poll status is pending
    let status_pending = client.poll_checkout_status(&checkout.checkout_id).expect("poll failed");
    assert_eq!(status_pending, "pending");

    // Simulate Mayar Webhook payment on Mock server
    {
        let mut checkouts = state.checkouts.lock().unwrap();
        let chk = checkouts.get_mut(&checkout.checkout_id).unwrap();
        chk.status = "paid".to_string();

        let mut accounts = state.accounts.lock().unwrap();
        let acc = accounts.values_mut().find(|a| a.id == chk.account_id).unwrap();
        acc.tier = "pro".to_string();
        acc.billing_type = "monthly".to_string();
        acc.status = "active".to_string();
        acc.expires_at = Some("2030-01-01T00:00:00Z".to_string());
    }

    // Poll status turns to paid
    let status_paid = client.poll_checkout_status(&checkout.checkout_id).expect("poll failed");
    assert_eq!(status_paid, "paid");

    // Client refreshes entitlement -> Instantly becomes Pro without restart
    let (payload_pro, state_pro) = client.refresh_entitlement().expect("refresh pro failed");
    assert_eq!(payload_pro.tier, "pro");
    assert_eq!(state_pro.tier, EffectiveTier::Pro);
    assert!(state_pro.is_pro);

    // =========================================================================
    // Scenario 2: USD Plan -> Completion Mode: "license_key" (Ko-fi)
    // =========================================================================
    // Client logs out / cleans state to simulate a clean international purchase
    store::clear_billing_state().unwrap();

    let usd_plans = client.get_plans(Some("USD")).expect("get USD plans failed");
    let lifetime_usd = usd_plans.iter().find(|p| p.billing_type == "lifetime").unwrap();

    // In Ko-fi flow, buyer purchases on Ko-fi and receives a license key via email
    let issued_key = "KOFI-LIFETIME-XYZ999";
    {
        let mut licenses = state.license_keys.lock().unwrap();
        licenses.insert(
            issued_key.to_string(),
            MockLicense {
                key: issued_key.to_string(),
                product_code: "CATERM".to_string(),
                tier: "pro".to_string(),
                billing_type: "lifetime".to_string(),
                revoked: false,
            },
        );
    }

    // Buyer enters the key in the app -> triggers activate_license
    let lic_auth = client
        .activate_license(issued_key, "dev-buyer-2", "MacBook Pro M3")
        .expect("license activation failed");
    assert!(lic_auth.account.email.contains("license_"));

    // Immediately becomes Lifetime Pro
    let (payload_lic, state_lic) = client.refresh_entitlement().expect("lic refresh failed");
    assert_eq!(payload_lic.tier, "pro");
    assert_eq!(payload_lic.billing_type, "lifetime");
    assert_eq!(state_lic.tier, EffectiveTier::Pro);
    assert!(state_lic.is_pro);
}
