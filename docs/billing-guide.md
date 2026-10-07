# Billing Hub (GCC) Integration Guide

This guide explains how desktop and mobile apps generated from **CAFramework** integrate with the central **GCC Billing Hub** (supporting Mayar.id for IDR and Ko-fi for USD).

---

## 1. Architecture Overview

- **Single Billing Hub:** Desktop and mobile applications never call Mayar or Ko-fi directly. All payments, accounts, device limits, and entitlements are managed centrally by GCC.
- **Offline & Zero-Trust Token:** When an account is verified, GCC issues a signed Ed25519 token (compact JWS `EdDSA`). The app verifies this token locally against public keys configured in `app.toml`.
- **Offline Allowance:** Entitlements stay valid offline for up to **7 days** after the last online check.
- **Dual Completion Modes:**
  - `poll`: Used for Mayar IDR checkouts. The app opens the Mayar URL and polls GCC until payment is settled, transitioning from Free to Pro immediately without restart.
  - `license_key`: Used for Ko-fi USD checkouts. The buyer receives a license key by email and enters it in the app.

---

## 2. Configuring `app.toml`

Every app generated with `cargo xtask new-app` contains a `[billing]` section:

```toml
[billing]
enabled = true
product_code = "CACASH"                  # Must match product code registered in GCC
gcc_base_url = "https://gcc.fathforce.com/api/billing"
offline_days = 7
refresh_hours = 6
currencies = ["IDR", "USD"]

[[billing.public_keys]]
kid = "gcc-2026-10"
ed25519 = "Su0nese5LuWHeNPFIz69p0HGUsdT49rnEn82fbtY2S8="
```

---

## 3. Running the Local Mock Server for Testing

During local development, you can run the mock GCC server provided by `caf-xtask`:

```bash
cargo xtask gcc-mock --port 8888
```

The mock server will:
1. Generate dev Ed25519 keys and print the public key.
2. Emulate OTP email delivery (printed to console).
3. Handle Mayar and Ko-fi checkout flows.
4. Expose admin endpoints to simulate upgrades, expiry, and key rotation.
