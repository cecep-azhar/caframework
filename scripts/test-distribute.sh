#!/usr/bin/env bash
set -euo pipefail

echo "========================================================"
echo "  TESTING UNIVERSAL MULTI-MIRROR RELEASE DISTRIBUTOR"
echo "========================================================"

TEST_DIR=$(mktemp -d /tmp/caf_distribute_test_XXXXXX)
trap 'rm -rf "$TEST_DIR"' EXIT

echo "--> Created mock artifact test directory: $TEST_DIR"

# 1. Create mock build artifacts (.exe, .apk, .deb, .rpm, .AppImage + .sig)
echo "test-exe-binary-content-12345" > "$TEST_DIR/caterm-v2.1.16-setup.exe"
echo "test-sig-windows-minisign" > "$TEST_DIR/caterm-v2.1.16-setup.exe.sig"

echo "test-appimage-binary-67890" > "$TEST_DIR/caterm-v2.1.16.AppImage"
echo "test-sig-linux-minisign" > "$TEST_DIR/caterm-v2.1.16.AppImage.sig"

echo "test-deb-package" > "$TEST_DIR/caterm_2.1.16_amd64.deb"
echo "test-rpm-package" > "$TEST_DIR/caterm-2.1.16-1.x86_64.rpm"
echo "test-apk-package" > "$TEST_DIR/caterm-v2.1.16-arm64-v8a.apk"

echo "--> Created 5 artifacts + 2 signature files."

# 2. Run distribute-release.sh in Dry Run mode
echo "--> Running distribute-release.sh in --dry-run mode..."

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$SCRIPT_DIR/distribute-release.sh" \
  --slug caterm \
  --tag v2.1.16 \
  --dir "$TEST_DIR" \
  --channel stable \
  --notes "Release test notes for v2.1.16" \
  --dry-run

echo "--> Verifying SHA256SUMS generated..."
if [[ -f "$TEST_DIR/SHA256SUMS" ]]; then
  echo "[PASS] SHA256SUMS exists and content:"
  cat "$TEST_DIR/SHA256SUMS"
else
  echo "[FAIL] SHA256SUMS not found"
  exit 1
fi

echo "--> Verifying latest.json updater manifest generated..."
if [[ -f "$TEST_DIR/latest.json" ]]; then
  echo "[PASS] latest.json exists and valid JSON:"
  jq . "$TEST_DIR/latest.json"
  
  # Validate key fields
  ver=$(jq -r .version "$TEST_DIR/latest.json")
  if [[ "$ver" != "2.1.16" ]]; then
    echo "[FAIL] Expected version 2.1.16, got $ver"
    exit 1
  fi
  
  win_sig=$(jq -r '.platforms["windows-x86_64"].signature' "$TEST_DIR/latest.json")
  linux_sig=$(jq -r '.platforms["linux-x86_64"].signature' "$TEST_DIR/latest.json")
  
  if [[ "$win_sig" != "test-sig-windows-minisign" ]]; then
    echo "[FAIL] Expected windows sig, got $win_sig"
    exit 1
  fi
  if [[ "$linux_sig" != "test-sig-linux-minisign" ]]; then
    echo "[FAIL] Expected linux sig, got $linux_sig"
    exit 1
  fi
else
  echo "[FAIL] latest.json not found"
  exit 1
fi

echo "========================================================"
echo "  ALL VALIDATION TESTS PASSED!"
echo "========================================================"
