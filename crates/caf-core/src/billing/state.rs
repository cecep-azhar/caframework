//! Effective Entitlement Plan computation based on cached token, local clock, and the 7 state rules.

use super::token::EntitlementPayload;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectiveTier {
    Free,
    Pro,
    ProGrace,
    FreeNeedsOnlineCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveState {
    pub tier: EffectiveTier,
    pub billing_type: String,
    pub is_pro: bool,
    pub expires_at: Option<String>,
    pub grace_until: Option<String>,
    pub warning: Option<String>,
}

const OFFLINE_LIMIT_SECS: u64 = 7 * 86400; // 7 days
const CLOCK_ROLLBACK_TOLERANCE_SECS: u64 = 600; // 10 minutes

/// Computes the effective plan following the 7 rules in PRD / Prompt:
/// 1. No valid token -> Free.
/// 2. Signature, product_code, or device_id mismatch -> Free (handled during token verify).
/// 3. status = revoked or expired -> Free.
/// 4. now > last_verified_at + 7 days -> Free (needs online check).
/// 5. Clock rollback (now < max_seen_time - 10 min) -> Free (needs online check).
/// 6. Lifetime -> Pro; Timed: now <= expires_at -> Pro; expires_at < now <= grace_until -> Pro (grace); after grace_until -> Free.
/// 7. A Free state never deletes user data; Pro features become read-only or hidden.
pub fn compute_effective_state(
    token_payload: Option<&EntitlementPayload>,
    last_verified_at: Option<u64>,
    max_seen_time: Option<u64>,
    now_unix: u64,
) -> EffectiveState {
    let payload = match token_payload {
        Some(p) => p,
        None => {
            return EffectiveState {
                tier: EffectiveTier::Free,
                billing_type: "free".to_string(),
                is_pro: false,
                expires_at: None,
                grace_until: None,
                warning: None,
            };
        }
    };

    // Rule 3: Revoked or Expired status directly from server
    if payload.status == "revoked" || payload.status == "expired" {
        return EffectiveState {
            tier: EffectiveTier::Free,
            billing_type: payload.billing_type.clone(),
            is_pro: false,
            expires_at: payload.expires_at.clone(),
            grace_until: payload.grace_until.clone(),
            warning: Some(format!("Entitlement is {}", payload.status)),
        };
    }

    // Rule 5: Clock rollback detection
    if let Some(max_time) = max_seen_time {
        if now_unix + CLOCK_ROLLBACK_TOLERANCE_SECS < max_time {
            return EffectiveState {
                tier: EffectiveTier::FreeNeedsOnlineCheck,
                billing_type: payload.billing_type.clone(),
                is_pro: false,
                expires_at: payload.expires_at.clone(),
                grace_until: payload.grace_until.clone(),
                warning: Some("Clock rollback detected. Please reconnect online.".into()),
            };
        }
    }

    // Rule 4: Offline allowance limit (7 days)
    if let Some(verified_at) = last_verified_at {
        if now_unix > verified_at + OFFLINE_LIMIT_SECS {
            return EffectiveState {
                tier: EffectiveTier::FreeNeedsOnlineCheck,
                billing_type: payload.billing_type.clone(),
                is_pro: false,
                expires_at: payload.expires_at.clone(),
                grace_until: payload.grace_until.clone(),
                warning: Some("Offline allowance exceeded (7 days). Please reconnect online.".into()),
            };
        }
    }

    // Rule 6: Lifetime or Timed Entitlement check
    if payload.tier == "free" || payload.billing_type == "free" {
        return EffectiveState {
            tier: EffectiveTier::Free,
            billing_type: "free".to_string(),
            is_pro: false,
            expires_at: None,
            grace_until: None,
            warning: None,
        };
    }

    if payload.billing_type == "lifetime" || payload.expires_at.is_none() {
        return EffectiveState {
            tier: EffectiveTier::Pro,
            billing_type: "lifetime".to_string(),
            is_pro: true,
            expires_at: None,
            grace_until: None,
            warning: None,
        };
    }

    // Parse expires_at and grace_until
    let expires_at_unix = payload
        .expires_at
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc).timestamp() as u64);

    let grace_until_unix = payload
        .grace_until
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc).timestamp() as u64);

    if let Some(exp) = expires_at_unix {
        if now_unix <= exp {
            // Active Pro
            EffectiveState {
                tier: EffectiveTier::Pro,
                billing_type: payload.billing_type.clone(),
                is_pro: true,
                expires_at: payload.expires_at.clone(),
                grace_until: payload.grace_until.clone(),
                warning: None,
            }
        } else if let Some(grace) = grace_until_unix {
            if now_unix <= grace {
                // In Grace Period (7 days)
                EffectiveState {
                    tier: EffectiveTier::ProGrace,
                    billing_type: payload.billing_type.clone(),
                    is_pro: true,
                    expires_at: payload.expires_at.clone(),
                    grace_until: payload.grace_until.clone(),
                    warning: Some("Plan expired. You are in a 7-day grace period.".into()),
                }
            } else {
                // Grace expired
                EffectiveState {
                    tier: EffectiveTier::Free,
                    billing_type: payload.billing_type.clone(),
                    is_pro: false,
                    expires_at: payload.expires_at.clone(),
                    grace_until: payload.grace_until.clone(),
                    warning: Some("Subscription expired.".into()),
                }
            }
        } else {
            // Expired without grace
            EffectiveState {
                tier: EffectiveTier::Free,
                billing_type: payload.billing_type.clone(),
                is_pro: false,
                expires_at: payload.expires_at.clone(),
                grace_until: payload.grace_until.clone(),
                warning: Some("Subscription expired.".into()),
            }
        }
    } else {
        EffectiveState {
            tier: EffectiveTier::Pro,
            billing_type: payload.billing_type.clone(),
            is_pro: true,
            expires_at: None,
            grace_until: None,
            warning: None,
        }
    }
}
