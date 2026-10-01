# Android App Signing Keystore Documentation

Dokumentasi konfigurasi release keystore permanen untuk signing aplikasi Android **CAFramework** (`com.fathforce.caframework`).

---

## 1. Keystore Details & Fingerprint

Keystore permanen telah dibuat menggunakan `keytool` (RSA 2048-bit, validitas 10.000 hari / ~27 tahun).

- **Keystore File Location**: `/home/cecepazhar/.android/caframework-release.keystore`
- **File Permissions**: `600` (`-rw-------`)
- **Key Alias**: `caframework-release`
- **Distinguished Name (dname)**: `CN=Cecep Saeful Azhar, O=Fathforce, C=ID`
- **Validity**: Fri Sep 25 2026 s/d Tue Feb 10 2054 (10,000 hari)
- **Key Algorithm**: RSA 2048-bit
- **Signature Algorithm**: `SHA384withRSA`

### Certificate Fingerprints
- **SHA-256**: `3A:CC:2C:7F:F2:7A:0D:C5:19:57:DD:80:91:B7:D7:DB:D2:2E:3F:D4:29:ED:83:E7:E2:86:12:8E:8E:CB:D4:52`
- **SHA-1**: `B2:0B:7D:8E:4A:CE:9D:09:A9:36:79:CF:85:E6:2F:20:67:C0:8F:D6`

---

## 2. Secure Storage & Backup Plan

1. **Local Storage**:
   - Disimpan di mesin lokal host: `/home/cecepazhar/.android/caframework-release.keystore` dengan permissions `chmod 600`.
2. **Offline & Remote Backup**:
   - Simpan salinan cadangan terenkripsi di Bitwarden / 1Password / Vault terproteksi. Kehilangan keystore ini akan menyebabkan aplikasi Android tidak dapat di-update (update signature mismatch).
3. **CI / CD (GitHub Actions Secrets)**:
   - Keystore dapat di-encode ke Base64 untuk disimpan sebagai secret GitHub Actions:
     ```bash
     base64 -w 0 /home/cecepazhar/.android/caframework-release.keystore > caframework-release.keystore.base64
     ```
   - Masukkan nilai base64 ke GitHub Repository Secret: `ANDROID_KEYSTORE_BASE64`
   - Masukkan password keystore/key ke GitHub Repository Secret: `ANDROID_KEYSTORE_PASSWORD` dan `ANDROID_KEY_ALIAS` (`caframework-release`).

---

## 3. Integrasi Build & Signing (Local / CI)

### Opsi A: Signing manual menggunakan `apksigner`
Jika build menghasilkan unsigned release APK (misal dari Gradle / Tauri CLI):
```bash
apksigner sign --ks /home/cecepazhar/.android/caframework-release.keystore \
  --ks-key-alias caframework-release \
  --in app-unsigned.apk \
  --out app-signed.apk
```

Verifikasi signing:
```bash
apksigner verify --verbose --print-certs app-signed.apk
```

### Opsi B: GitHub Actions Workflow Step
```yaml
- name: Decode Android Keystore
  run: |
    echo "${{ secrets.ANDROID_KEYSTORE_BASE64 }}" | base64 -d > /tmp/caframework-release.keystore

- name: Sign Android APK
  run: |
    $ANDROID_HOME/build-tools/34.0.0/apksigner sign \
      --ks /tmp/caframework-release.keystore \
      --ks-key-alias ${{ secrets.ANDROID_KEY_ALIAS || 'caframework-release' }} \
      --ks-pass pass:${{ secrets.ANDROID_KEYSTORE_PASSWORD }} \
      --key-pass pass:${{ secrets.ANDROID_KEYSTORE_PASSWORD }} \
      --out dist/caframework-v${{ github.ref_name }}-signed.apk \
      path/to/unsigned.apk
```
