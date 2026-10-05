use caf_xtask::gcc_mock::{create_mock_app, init_dev_keys, MockState};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};

#[tokio::test]
async fn test_gcc_mock_contract_compliance() {
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

    let app = create_mock_app(state);

    // 1. Test OTP Request
    let req = Request::builder()
        .method("POST")
        .uri("/v1/auth/otp/request")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"pro@example.com","product_code":"CACASH"}"#))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);

    // 2. Test OTP Verify
    let req = Request::builder()
        .method("POST")
        .uri("/v1/auth/otp/verify")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"email":"pro@example.com","otp_code":"123456","product_code":"CACASH","device_id":"dev-123","device_name":"X1 Laptop"}"#))
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    
    let body_bytes = axum::body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let val: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let device_token = val["device_token"].as_str().unwrap().to_string();

    // 3. Test Entitlement
    let req = Request::builder()
        .method("GET")
        .uri("/v1/entitlement?product_code=CACASH")
        .header("authorization", format!("Bearer {}", device_token))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 4. Test Plans
    let req = Request::builder()
        .method("GET")
        .uri("/v1/plans?product_code=CACASH&currency=IDR")
        .header("authorization", format!("Bearer {}", device_token))
        .body(Body::empty())
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 5. Test Public Keys
    let req = Request::builder()
        .method("GET")
        .uri("/.well-known/gcc-billing-keys")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
