//! Feature-level Pro access guards.

use super::client::GccBillingClient;
use super::state::{compute_effective_state, EffectiveState, EffectiveTier};
use super::store;
use super::token::verify_and_parse_token;
use crate::error::{CatermError, ProError};

/// Checks if the current cached billing state grants access to a Pro feature.
/// Returns Ok(EffectiveState) if Pro or ProGrace, otherwise returns Err(ProError::FeatureLocked).
pub fn require_pro(
    public_keys: &[(String, String)],
    product_code: &str,
    now_unix: u64,
) -> Result<EffectiveState, CatermError> {
    let device_id = store::get_device_id()?.unwrap_or_else(|| "default_dev".to_string());
    let cached_token = store::get_cached_token()?;
    let last_verified_at = store::get_last_verified_at()?;
    let max_seen_time = store::get_max_seen_time()?;

    let token_payload = if let Some(ref tok) = cached_token {
        verify_and_parse_token(tok, public_keys, product_code, &device_id).ok()
    } else {
        None
    };

    let effective = compute_effective_state(
        token_payload.as_ref(),
        last_verified_at,
        max_seen_time,
        now_unix,
    );

    if effective.is_pro && (effective.tier == EffectiveTier::Pro || effective.tier == EffectiveTier::ProGrace) {
        Ok(effective)
    } else {
        Err(CatermError::Pro(ProError::FeatureLocked))
    }
}
