#!/usr/bin/env bash
set -euo pipefail

# CAFramework v2 - Simple Build All Script (Linux deb/rpm/appimage & Windows x64)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"

TARGET="${1:-all}"

echo "========================================"
echo "  CAFramework v2 Multi-Platform Builder"
echo "  Target: $TARGET"
echo "========================================"

mkdir -p "$DIST_DIR"

# 1. Build Frontend SvelteKit
echo "--> [1/3] Building SvelteKit Frontend..."
cd "$ROOT_DIR/frontend"
npm install
npm run build

# 2. Build Linux Bundles (deb, rpm, AppImage)
if [[ "$TARGET" == "all" || "$TARGET" == "linux" ]]; then
  echo "--> [2/3] Building Linux Bundles (.deb, .rpm, .AppImage) via Podman..."
  cd "$ROOT_DIR"
  podman run --rm -v "$ROOT_DIR":/app:z -w /app rust:latest bash -c "
    set -e
    apt-get update -qq
    apt-get install -y -qq nodejs npm libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libayatana-appindicator3-dev librsvg2-dev rpm alien
    npm install -g @tauri-apps/cli
    cd /app/crates/caf-app
    npx tauri build

    # --- Icon workaround -------------------------------------------------
    # tauri-bundler's native rpm target does not package the hicolor icon
    # set (usr/share/icons/hicolor/**/apps/caf-app.png) into the .rpm —
    # confirmed by diffing the deb vs rpm bundle trees: the .deb DOES
    # include those icon files, the .rpm ships only the .desktop entry.
    # Result: after installing the plain tauri rpm, the .desktop's
    # Icon=caf-app can't be resolved by the icon theme, so the taskbar /
    # app switcher falls back to a generic icon.
    #
    # Fix: rebuild the rpm from the already-correct .deb via alien, which
    # carries the icon files over, instead of shipping tauri's icon-less
    # rpm output.
    RPM_DIR=/app/target/release/bundle/rpm
    DEB_DIR=/app/target/release/bundle/deb
    REPAIRED_DIR=/app/target/release/bundle/rpm-repaired
    mkdir -p \"\$REPAIRED_DIR\"
    for deb in \"\$DEB_DIR\"/*.deb; do
      [ -e \"\$deb\" ] || continue
      echo \"--> Repairing rpm icon via alien from: \$(basename \"\$deb\")\"
      ( cd \"\$REPAIRED_DIR\" && alien --to-rpm --scripts -k \"\$deb\" ) || echo \"WARN: alien conversion failed for \$deb, falling back to tauri's rpm\"
    done
  "

  # Copy Linux artifacts to dist/
  cp -f "$ROOT_DIR"/target/release/bundle/deb/*.deb "$DIST_DIR/" 2>/dev/null || true
  # Prefer the icon-repaired rpm (built from the deb via alien); fall back
  # to tauri's native rpm only if the alien repair step failed.
  if ls "$ROOT_DIR"/target/release/bundle/rpm-repaired/*.rpm >/dev/null 2>&1; then
    cp -f "$ROOT_DIR"/target/release/bundle/rpm-repaired/*.rpm "$DIST_DIR/"
  else
    cp -f "$ROOT_DIR"/target/release/bundle/rpm/*.rpm "$DIST_DIR/" 2>/dev/null || true
  fi
  cp -f "$ROOT_DIR"/target/release/bundle/appimage/*.AppImage "$DIST_DIR/" 2>/dev/null || true
fi

# 3. Build Windows x64 (.exe)
if [[ "$TARGET" == "all" || "$TARGET" == "win" || "$TARGET" == "windows" ]]; then
  echo "--> [3/3] Cross-compiling Windows x64 (.exe) via Podman..."
  cd "$ROOT_DIR"
  podman run --rm -v "$ROOT_DIR":/app:z -w /app rust:latest bash -c "
    set -e
    apt-get update -qq
    apt-get install -y -qq mingw-w64 pkg-config
    rustup target add x86_64-pc-windows-gnu
    cargo build --target x86_64-pc-windows-gnu -p caf-app --release
  "

  # Copy Windows artifact to dist/
  cp -f "$ROOT_DIR/target/x86_64-pc-windows-gnu/release/caf-app.exe" "$DIST_DIR/CAFramework-x64.exe" 2>/dev/null || true
fi

echo ""
echo "========================================"
echo "  Build Completed! Artifacts in dist/:"
echo "========================================"
ls -lh "$DIST_DIR"
