use caf_core::billing::gate::require_pro;
use caf_core::billing::store;
use caf_core::billing::token::EntitlementPayload;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use rand::RngCore;

fn generate_keys() -> (SigningKey, String, String) {
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let pub_key_b64 = base64::engine::general_purpose::STANDARD
        .encode(signing_key.verifying_key().as_bytes());
    let kid = "gcc-2026-10".to_string();
    (signing_key, pub_key_b64, kid)
}

fn create_token(payload: &EntitlementPayload, signing_key: &SigningKey, kid: &str) -> String {
    let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&serde_json::json!({"alg": "EdDSA", "typ": "JWT", "kid": kid})).unwrap());
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(payload).unwrap());
    let signing_input = format!("{}.{}", header_b64, payload_b64);
    let sig_bytes = signing_key.sign(signing_input.as_bytes()).to_bytes();
    let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig_bytes);
    format!("{}.{}", signing_input, sig_b64)
}

#[test]
fn test_require_pro_rust_guard_rejection_and_acceptance() {
    let (key, pub_b64, kid) = generate_keys();
    let keys = vec![(kid.clone(), pub_b64)];

    // 1. When no token stored -> Free state -> require_pro returns Err(FeatureLocked)
    store::clear_billing_state().unwrap();
    let res = require_pro(&keys, "CACASH", 1000);
    assert!(res.is_err());

    // 2. When valid Pro token is stored -> require_pro returns Ok(EffectiveState)
    let payload = EntitlementPayload {
        v: 1,
        kid: kid.clone(),
        account_id: "acc-1".into(),
        product_code: "CACASH".into(),
        device_id: "dev-gate-1".into(),
        tier: "pro".into(),
        billing_type: "lifetime".into(),
        status: "active".into(),
        expires_at: None,
        grace_until: None,
        issued_at: "2026-10-05T00:00:00Z".into(),
    };

    let token = create_token(&payload, &key, &kid);
    store::save_device_info("dev-gate-1", "tok_dev_1").unwrap();
    store::save_cached_token(&token, 1000).unwrap();

    let res_ok = require_pro(&keys, "CACASH", 1000).expect("require_pro should succeed");
    assert!(res_ok.is_pro);
    assert_eq!(res_ok.billing_type, "lifetime");
}
