//! HTTP Client for GCC Billing Hub endpoints.

use super::state::{compute_effective_state, EffectiveState};
use super::store;
use super::token::{verify_and_parse_token, EntitlementPayload};
use crate::error::{CatermError, VaultError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingConfig {
    pub enabled: bool,
    pub product_code: String,
    pub gcc_base_url: String,
    pub public_keys: Vec<(String, String)>, // (kid, base64_pubkey)
    pub offline_days: u32,
    pub refresh_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthAccount {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub device_token: String,
    pub account: AuthAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub plan_id: String,
    pub billing_type: String,
    pub price_minor: u64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub checkout_id: String,
    pub checkout_url: String,
    pub completion_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutStatusResponse {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceItem {
    pub id: String,
    pub device_id: String,
    pub device_name: String,
    pub is_current: bool,
    pub last_seen_at: String,
}

pub struct GccBillingClient {
    config: BillingConfig,
}

impl GccBillingClient {
    pub fn new(config: BillingConfig) -> Self {
        Self { config }
    }

    /// Step 1: Request OTP email
    pub fn request_otp(&self, email: &str) -> Result<(), CatermError> {
        let url = format!("{}/v1/auth/otp/request", self.config.gcc_base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "email": email,
            "product_code": self.config.product_code
        });

        let mut resp = ureq::post(&url)
            .send_json(&body)
            .map_err(|e| map_ureq_err("request_otp", e))?;

        if resp.status().as_u16() == 202 {
            Ok(())
        } else {
            let body_str = resp.body_mut().read_to_string().unwrap_or_default();
            Err(CatermError::Vault(VaultError::Generic(format!(
                "Failed to request OTP (HTTP {}): {}",
                resp.status(),
                body_str
            ))))
        }
    }

    /// Step 2: Verify OTP and register device
    pub fn verify_otp(
        &self,
        email: &str,
        otp_code: &str,
        device_id: &str,
        device_name: &str,
    ) -> Result<AuthResponse, CatermError> {
        let url = format!("{}/v1/auth/otp/verify", self.config.gcc_base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "email": email,
            "otp_code": otp_code,
            "product_code": self.config.product_code,
            "device_id": device_id,
            "device_name": device_name
        });

        let mut resp = ureq::post(&url)
            .send_json(&body)
            .map_err(|e| map_ureq_err("verify_otp", e))?;

        let auth: AuthResponse = resp
            .body_mut()
            .read_json()
            .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid JSON response: {e}"))))?;

        store::save_device_info(device_id, &auth.device_token)?;
        Ok(auth)
    }

    /// Activate license key on device (Fallback / Ko-fi / Admin issued)
    pub fn activate_license(
        &self,
        license_key: &str,
        device_id: &str,
        device_name: &str,
    ) -> Result<AuthResponse, CatermError> {
        let url = format!("{}/v1/licenses/activate", self.config.gcc_base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "license_key": license_key,
            "product_code": self.config.product_code,
            "device_id": device_id,
            "device_name": device_name
        });

        let mut resp = ureq::post(&url)
            .send_json(&body)
            .map_err(|e| map_ureq_err("activate_license", e))?;

        let auth: AuthResponse = resp
            .body_mut()
            .read_json()
            .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid JSON response: {e}"))))?;

        store::save_device_info(device_id, &auth.device_token)?;
        Ok(auth)
    }

    /// Refresh and verify current signed entitlement token
    pub fn refresh_entitlement(&self) -> Result<(EntitlementPayload, EffectiveState), CatermError> {
        let device_token = match store::get_device_token()? {
            Some(t) => t,
            None => {
                return Err(CatermError::Vault(VaultError::Generic(
                    "No device token found. Please log in.".into(),
                )));
            }
        };
        let device_id = store::get_device_id()?.unwrap_or_else(|| "default_dev".to_string());

        let url = format!(
            "{}/v1/entitlement?product_code={}",
            self.config.gcc_base_url.trim_end_matches('/'),
            self.config.product_code
        );

        let mut resp = ureq::get(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .call()
            .map_err(|e| map_ureq_err("refresh_entitlement", e))?;

        #[derive(Deserialize)]
        struct TokenResp {
            token: String,
        }

        let body: TokenResp = resp
            .body_mut()
            .read_json()
            .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid JSON: {e}"))))?;

        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Verify and parse the signed JWT
        let payload = verify_and_parse_token(
            &body.token,
            &self.config.public_keys,
            &self.config.product_code,
            &device_id,
        )?;

        // Cache in SQLCipher
        store::save_cached_token(&body.token, now_unix)?;

        let max_seen = store::get_max_seen_time()?;
        let effective = compute_effective_state(Some(&payload), Some(now_unix), max_seen, now_unix);

        Ok((payload, effective))
    }

    /// Fetch list of plans
    pub fn get_plans(&self, currency: Option<&str>) -> Result<Vec<PlanItem>, CatermError> {
        let device_token = store::get_device_token()?.unwrap_or_default();
        let mut url = format!(
            "{}/v1/plans?product_code={}",
            self.config.gcc_base_url.trim_end_matches('/'),
            self.config.product_code
        );
        if let Some(curr) = currency {
            url.push_str(&format!("&currency={}", curr));
        }

        let mut resp = ureq::get(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .call()
            .map_err(|e| map_ureq_err("get_plans", e))?;

        let plans: Vec<PlanItem> = resp
            .body_mut()
            .read_json()
            .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid JSON: {e}"))))?;

        Ok(plans)
    }

    /// Create Checkout URL
    pub fn create_checkout(&self, plan_id: &str) -> Result<CheckoutResponse, CatermError> {
        let device_token = match store::get_device_token()? {
            Some(t) => t,
            None => {
                return Err(CatermError::Vault(VaultError::Generic(
                    "Device token required to checkout".into(),
                )));
            }
        };

        let url = format!("{}/v1/checkout", self.config.gcc_base_url.trim_end_matches('/'));
        let body = serde_json::json!({ "plan_id": plan_id });

        let mut resp = ureq::post(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .send_json(&body)
            .map_err(|e| map_ureq_err("create_checkout", e))?;

        let res: CheckoutResponse = resp
            .body_mut()
            .read_json()
            .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid JSON: {e}"))))?;

        Ok(res)
    }

    /// Poll Checkout Status
    pub fn poll_checkout_status(&self, checkout_id: &str) -> Result<String, CatermError> {
        let device_token = store::get_device_token()?.unwrap_or_default();
        let url = format!(
            "{}/v1/checkout/{}",
            self.config.gcc_base_url.trim_end_matches('/'),
            checkout_id
        );

        let mut resp = ureq::get(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .call()
            .map_err(|e| map_ureq_err("poll_checkout_status", e))?;

        let st: CheckoutStatusResponse = resp
            .body_mut()
            .read_json()
            .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid JSON: {e}"))))?;

        Ok(st.status)
    }

    /// List registered devices
    pub fn list_devices(&self) -> Result<Vec<DeviceItem>, CatermError> {
        let device_token = match store::get_device_token()? {
            Some(t) => t,
            None => return Ok(vec![]),
        };

        let url = format!("{}/v1/devices", self.config.gcc_base_url.trim_end_matches('/'));
        let mut resp = ureq::get(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .call()
            .map_err(|e| map_ureq_err("list_devices", e))?;

        let items: Vec<DeviceItem> = resp
            .body_mut()
            .read_json()
            .map_err(|e| CatermError::Vault(VaultError::Generic(format!("invalid JSON: {e}"))))?;

        Ok(items)
    }

    /// Deactivate device / Sign out
    pub fn deactivate_device(&self, device_id: &str) -> Result<(), CatermError> {
        let device_token = match store::get_device_token()? {
            Some(t) => t,
            None => return Ok(()),
        };

        let url = format!(
            "{}/v1/devices/{}",
            self.config.gcc_base_url.trim_end_matches('/'),
            device_id
        );
        let _ = ureq::delete(&url)
            .header("Authorization", format!("Bearer {}", device_token))
            .call();

        store::clear_billing_state()?;
        Ok(())
    }
}

fn map_ureq_err(context: &str, err: ureq::Error) -> CatermError {
    CatermError::Vault(VaultError::Generic(format!(
        "GCC Billing Request Failed on {context}: {err}"
    )))
}
