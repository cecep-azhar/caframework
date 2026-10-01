# Creating a New Application with CAFramework

CAFramework menyediakan tool bawaan `cargo xtask new-app` untuk membuat aplikasi baru secara instan.

---

## 1. Menyiapkan Konfigurasi `app.toml`
Buat berkas spesifikasi aplikasi, misalnya `my-app.toml`:

```toml
[app]
name = "MyApp"
slug = "my-app"
bundle_id = "com.fathforce.myapp"
version = "0.1.0"
accent_color = "#3b82f6"
error_prefix = "MYAPP"
description = "My awesome desktop application"

[window]
title = "MyApp Desktop"
width = 1200
height = 800
min_width = 800
min_height = 600

[security]
master_password_required = true
profile_pin_enabled = true

[modules]
sample_slice_enabled = true
ai_assistant_enabled = true
pro_licensing_enabled = false
updater_enabled = false
crash_reporting_enabled = true
backup_restore_enabled = true

[roles]
default_role = "user"
allowed_roles = ["admin", "user"]

[[menus]]
key = "dashboard"
label = "Dashboard"
href = "/dashboard"
icon = "chart"
description = "Main dashboard view"
```

---

## 2. Menjalankan Generator Scaffolding
Jalankan perintah:
```bash
cargo run --package caf-xtask -- new-app --config my-app.toml --out ~/Project/my-app
```

Generator akan:
1. Menyalin struktur direktori dan template core.
2. Mengubah nama modul, identifier, error prefix, dan bundle ID.
3. Melakukan codegen frontend navigation & config otomatis.
4. Menyiapkan frontend build assets.
