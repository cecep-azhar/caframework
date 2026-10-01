# Android Development & Build Guide for CAFramework

Panduan komprehensif konfigurasi, build, signing, dan deployment aplikasi **CAFramework** dan produk turunannya pada platform Android via Tauri v2.

---

## 1. Arsitektur Android di CAFramework
- **Engine**: Tauri v2 Mobile Architecture (Rust Backend + WebView Frontend Svelte 5).
- **Package Name / Bundle ID**: `com.fathforce.caframework` (didefinisikan di `app.toml` & `tauri.conf.json`).
- **Database Engine**: SQLCipher cross-compiled untuk Android ABI (`arm64-v8a`, `armeabi-v7a`, `x86_64`).
- **Security**: Local master password hashing (Argon2id) + biometric/PIN support.

---

## 2. Prasyarat Lingkungan (Prerequisites)
1. **Rust & Android Targets**:
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
   ```
2. **Android SDK & NDK**:
   - `ANDROID_HOME` / `ANDROID_SDK_ROOT` diarahkan ke direktori Android SDK.
   - NDK versi `26.x` atau lebih baru.
   - Java JDK 17 atau 21 (`JAVA_HOME`).
3. **Node.js & Tauri CLI**:
   ```bash
   npm install -g @tauri-apps/cli@latest
   ```

---

## 3. Inisialisasi Project Android (Tauri Mobile)
Untuk menginisialisasi binding Android pertama kali:
```bash
cd /home/cecepazhar/Project/caframework/crates/caf-app
cargo tauri android init
```

Perintah ini akan membuat subdirektori `gen/android/` yang berisi Android Studio project dan Gradle build scripts.

---

## 4. Konfigurasi Signing Keystore
Untuk build release APK / AAB yang siap dipublikasikan ke Google Play Store atau didistribusikan langsung:

1. Pastikan keystore telah dibuat atau gunakan keystore default Fathforce:
   - Path: `~/.android/caframework-release.keystore`
   - Alias: `caframework-release`
2. Konfigurasi `gen/android/app/build.gradle.kts` atau export environment variables:
   ```bash
   export ANDROID_KEYSTORE_PATH="/home/cecepazhar/.android/caframework-release.keystore"
   export ANDROID_KEY_ALIAS="caframework-release"
   ```

---

## 5. Menjalankan & Build APK / App Bundle

### Mode Development (Hot-Reload di Emulator / Device)
```bash
cargo tauri android dev
```

### Mode Release Build (APK & AAB)
```bash
# Build APK standalone
cargo tauri android build --apk

# Build AAB untuk Google Play
cargo tauri android build --aab
```

Artifacts hasil build akan berada di:
- `gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`
- `gen/android/app/build/outputs/bundle/release/app-release.aab`
