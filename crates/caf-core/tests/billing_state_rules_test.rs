use caf_core::billing::state::{compute_effective_state, EffectiveTier};
use caf_core::billing::token::{verify_and_parse_token, EntitlementPayload};
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use rand::RngCore;

fn generate_test_keys() -> (SigningKey, String, String) {
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let pub_key_b64 = base64::engine::general_purpose::STANDARD
        .encode(signing_key.verifying_key().as_bytes());
    let kid = "gcc-2026-10".to_string();
    (signing_key, pub_key_b64, kid)
}

fn create_signed_token(
    payload: &EntitlementPayload,
    signing_key: &SigningKey,
    kid: &str,
    tamper_sig: bool,
) -> String {
    let header_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&serde_json::json!({"alg": "EdDSA", "typ": "JWT", "kid": kid})).unwrap());
    let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(payload).unwrap());
    let signing_input = format!("{}.{}", header_b64, payload_b64);

    let sig_bytes = if tamper_sig {
        [0u8; 64]
    } else {
        signing_key.sign(signing_input.as_bytes()).to_bytes()
    };
    let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sig_bytes);
    format!("{}.{}", signing_input, sig_b64)
}

#[test]
fn test_state_rule_1_no_token() {
    let state = compute_effective_state(None, None, None, 1700000000);
    assert_eq!(state.tier, EffectiveTier::Free);
    assert!(!state.is_pro);
}

#[test]
fn test_state_rule_2_bad_signature_or_mismatch() {
    let (key, pub_b64, kid) = generate_test_keys();
    let keys = vec![(kid.clone(), pub_b64)];

    let payload = EntitlementPayload {
        v: 1,
        kid: kid.clone(),
        account_id: "acc_1".into(),
        product_code: "CACASH".into(),
        device_id: "dev_1".into(),
        tier: "pro".into(),
        billing_type: "monthly".into(),
        status: "active".into(),
        expires_at: Some("2026-11-05T00:00:00Z".into()),
        grace_until: Some("2026-11-12T00:00:00Z".into()),
        issued_at: "2026-10-05T00:00:00Z".into(),
    };

    // Bad signature
    let bad_tok = create_signed_token(&payload, &key, &kid, true);
    assert!(verify_and_parse_token(&bad_tok, &keys, "CACASH", "dev_1").is_err());

    // Product mismatch
    let good_tok = create_signed_token(&payload, &key, &kid, false);
    assert!(verify_and_parse_token(&good_tok, &keys, "WRONG_PROD", "dev_1").is_err());

    // Device mismatch
    assert!(verify_and_parse_token(&good_tok, &keys, "CACASH", "dev_wrong").is_err());
}

#[test]
fn test_state_rule_3_revoked_or_expired_status() {
    let mut payload = EntitlementPayload {
        v: 1,
        kid: "k1".into(),
        account_id: "acc_1".into(),
        product_code: "CACASH".into(),
        device_id: "dev_1".into(),
        tier: "pro".into(),
        billing_type: "monthly".into(),
        status: "revoked".into(),
        expires_at: Some("2026-11-05T00:00:00Z".into()),
        grace_until: None,
        issued_at: "2026-10-05T00:00:00Z".into(),
    };

    let state = compute_effective_state(Some(&payload), Some(100), Some(100), 150);
    assert_eq!(state.tier, EffectiveTier::Free);
    assert!(!state.is_pro);

    payload.status = "expired".into();
    let state = compute_effective_state(Some(&payload), Some(100), Some(100), 150);
    assert_eq!(state.tier, EffectiveTier::Free);
    assert!(!state.is_pro);
}

#[test]
fn test_state_rule_4_offline_allowance_6_vs_8_days() {
    let payload = EntitlementPayload {
        v: 1,
        kid: "k1".into(),
        account_id: "acc_1".into(),
        product_code: "CACASH".into(),
        device_id: "dev_1".into(),
        tier: "pro".into(),
        billing_type: "monthly".into(),
        status: "active".into(),
        expires_at: Some("2030-01-01T00:00:00Z".into()),
        grace_until: None,
        issued_at: "2026-10-05T00:00:00Z".into(),
    };

    let verified_at = 1_000_000;
    // 6 days offline (6 * 86400 = 518400) -> Still Pro
    let now_6_days = verified_at + 518400;
    let state = compute_effective_state(Some(&payload), Some(verified_at), Some(now_6_days), now_6_days);
    assert_eq!(state.tier, EffectiveTier::Pro);
    assert!(state.is_pro);

    // 8 days offline (8 * 86400 = 691200) -> Needs Online Check
    let now_8_days = verified_at + 691200;
    let state = compute_effective_state(Some(&payload), Some(verified_at), Some(now_8_days), now_8_days);
    assert_eq!(state.tier, EffectiveTier::FreeNeedsOnlineCheck);
    assert!(!state.is_pro);
}

#[test]
fn test_state_rule_5_clock_rollback() {
    let payload = EntitlementPayload {
        v: 1,
        kid: "k1".into(),
        account_id: "acc_1".into(),
        product_code: "CACASH".into(),
        device_id: "dev_1".into(),
        tier: "pro".into(),
        billing_type: "monthly".into(),
        status: "active".into(),
        expires_at: Some("2030-01-01T00:00:00Z".into()),
        grace_until: None,
        issued_at: "2026-10-05T00:00:00Z".into(),
    };

    let max_seen = 2_000_000;
    // Clock moved backwards by 1 hour (3600s > 600s tolerance)
    let rolled_back_now = max_seen - 3600;
    let state = compute_effective_state(Some(&payload), Some(rolled_back_now), Some(max_seen), rolled_back_now);
    assert_eq!(state.tier, EffectiveTier::FreeNeedsOnlineCheck);
    assert!(!state.is_pro);
}

#[test]
fn test_state_rule_6_lifetime_and_grace_period() {
    // 1. Lifetime
    let lifetime_payload = EntitlementPayload {
        v: 1,
        kid: "k1".into(),
        account_id: "acc_1".into(),
        product_code: "CACASH".into(),
        device_id: "dev_1".into(),
        tier: "pro".into(),
        billing_type: "lifetime".into(),
        status: "active".into(),
        expires_at: None,
        grace_until: None,
        issued_at: "2026-10-05T00:00:00Z".into(),
    };
    let state = compute_effective_state(Some(&lifetime_payload), Some(100), Some(100), 200);
    assert_eq!(state.tier, EffectiveTier::Pro);
    assert!(state.is_pro);

    // 2. Timed: Active Pro -> Grace -> Expired
    let exp_str = "2026-11-05T00:00:00Z"; // 1793836800 unix approx
    let grace_str = "2026-11-12T00:00:00Z";
    let exp_unix = chrono::DateTime::parse_from_rfc3339(exp_str).unwrap().timestamp() as u64;
    let grace_unix = chrono::DateTime::parse_from_rfc3339(grace_str).unwrap().timestamp() as u64;

    let timed_payload = EntitlementPayload {
        v: 1,
        kid: "k1".into(),
        account_id: "acc_1".into(),
        product_code: "CACASH".into(),
        device_id: "dev_1".into(),
        tier: "pro".into(),
        billing_type: "monthly".into(),
        status: "active".into(),
        expires_at: Some(exp_str.into()),
        grace_until: Some(grace_str.into()),
        issued_at: "2026-10-05T00:00:00Z".into(),
    };

    // Before expiry -> Pro
    let state_active = compute_effective_state(Some(&timed_payload), Some(exp_unix - 1000), Some(exp_unix - 1000), exp_unix - 1000);
    assert_eq!(state_active.tier, EffectiveTier::Pro);
    assert!(state_active.is_pro);

    // During grace -> ProGrace
    let state_grace = compute_effective_state(Some(&timed_payload), Some(exp_unix + 1000), Some(exp_unix + 1000), exp_unix + 1000);
    assert_eq!(state_grace.tier, EffectiveTier::ProGrace);
    assert!(state_grace.is_pro);

    // After grace -> Free
    let state_expired = compute_effective_state(Some(&timed_payload), Some(grace_unix + 1000), Some(grace_unix + 1000), grace_unix + 1000);
    assert_eq!(state_expired.tier, EffectiveTier::Free);
    assert!(!state_expired.is_pro);
}
