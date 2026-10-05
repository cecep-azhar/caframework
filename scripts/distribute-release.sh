#!/usr/bin/env bash
# ==============================================================================
# Universal Multi-Mirror Release Distribution Script (caf-distribute)
# Targets:
#  1. GitHub Releases (`gh release upload`)
#  2. Cloudflare R2 (S3-compatible via AWS CLI or native curl/REST S3 SigV4)
#  3. YPC MinIO S3 (`s3.ypc.my.id` S3-compatible)
#  4. GCC Release Asset API (`/api/v1/releases/<slug>/versions/<tag>/assets`)
# Features:
#  - SHA-256 calculation per artifact & SHA256SUMS file generation
#  - Parallel uploading across destinations and artifacts
#  - Resilient retry with exponential backoff
#  - Updater manifest generation (Tauri v2 latest.json format)
#  - Dry run / validation test mode
# ==============================================================================

set -euo pipefail

VERSION="1.0.0"

# Defaults & Environment Fallbacks
SLUG="${RELEASE_SLUG:-}"
TAG="${RELEASE_TAG:-}"
DIST_DIR="${RELEASE_DIST_DIR:-dist}"
NOTES="${RELEASE_NOTES:-}"
CHANNEL="${RELEASE_CHANNEL:-stable}"
NOTIFY="${RELEASE_NOTIFY:-false}"
MAX_RETRIES="${RELEASE_MAX_RETRIES:-3}"
PARALLEL_JOBS="${RELEASE_PARALLEL_JOBS:-4}"
DRY_RUN=false

# Targets to enable
ENABLE_GITHUB=true
ENABLE_R2=true
ENABLE_YPC=true
ENABLE_GCC=true
ENABLE_MANIFEST=true

# GCC API Configuration
GCC_BASE_URL="${GCC_BASE_URL:-https://gcc.fathforce.com}"
GCC_API_KEY="${GCC_API_KEY:-}"

# GitHub Config
GITHUB_REPO="${GITHUB_REPOSITORY:-}"

# Cloudflare R2 S3 Config
R2_ENDPOINT="${R2_ENDPOINT:-}"
R2_BUCKET="${R2_BUCKET:-}"
R2_ACCESS_KEY_ID="${R2_ACCESS_KEY_ID:-}"
R2_SECRET_ACCESS_KEY="${R2_SECRET_ACCESS_KEY:-}"
R2_REGION="${R2_REGION:-auto}"
R2_PREFIX="${R2_PREFIX:-}"

# YPC MinIO S3 Config
YPC_ENDPOINT="${YPC_ENDPOINT:-https://s3.ypc.my.id}"
YPC_BUCKET="${YPC_BUCKET:-}"
YPC_ACCESS_KEY_ID="${YPC_ACCESS_KEY_ID:-}"
YPC_SECRET_ACCESS_KEY="${YPC_SECRET_ACCESS_KEY:-}"
YPC_REGION="${YPC_REGION:-auto}"
YPC_PREFIX="${YPC_PREFIX:-}"

# Color output helpers
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_ok() { echo -e "${GREEN}[OK]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
log_err() { echo -e "${RED}[ERROR]${NC} $*" >&2; }
log_step() { echo -e "\n${CYAN}==> $*${NC}"; }

usage() {
  cat << 'EOF'
Usage: distribute-release.sh [OPTIONS]

Universal multi-mirror release distribution script.
Uploads .exe, .apk, .deb, .rpm, .AppImage and updater signatures to:
GitHub Releases, Cloudflare R2, YPC MinIO S3, and GCC Storage API.

Options:
  -s, --slug SLUG          App slug (e.g. caframework, caterm) [REQUIRED or env RELEASE_SLUG]
  -t, --tag TAG            Release tag version (e.g. v0.1.0, v2.1.16) [REQUIRED or env RELEASE_TAG]
  -d, --dir DIRECTORY      Directory containing build artifacts [default: dist]
  -n, --notes NOTES        Release notes text or file path
  --channel CHANNEL        Release channel (stable | beta) [default: stable]
  --notify                 Trigger subscriber notifications via GCC [default: false]
  --dry-run                Simulate distribution without uploading
  --no-github              Skip GitHub Releases upload
  --no-r2                  Skip Cloudflare R2 upload
  --no-ypc                 Skip YPC MinIO S3 upload
  --no-gcc                 Skip GCC Release Asset API upload
  --no-manifest            Skip updater manifest (latest.json) generation
  -j, --jobs NUM           Max concurrent upload tasks [default: 4]
  -r, --retries NUM        Max retry attempts per operation [default: 3]
  -h, --help               Show this help message
  -v, --version            Show script version

Environment Variables:
  RELEASE_SLUG, RELEASE_TAG, RELEASE_DIST_DIR, RELEASE_NOTES, RELEASE_CHANNEL, RELEASE_NOTIFY
  GCC_BASE_URL, GCC_API_KEY
  GITHUB_REPOSITORY, GH_TOKEN
  R2_ENDPOINT, R2_BUCKET, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, R2_REGION, R2_PREFIX
  YPC_ENDPOINT, YPC_BUCKET, YPC_ACCESS_KEY_ID, YPC_SECRET_ACCESS_KEY, YPC_REGION, YPC_PREFIX

Examples:
  ./scripts/distribute-release.sh --slug caterm --tag v2.1.16 --dir dist
  ./scripts/distribute-release.sh --slug caframework --tag v0.1.0 --dry-run
EOF
}

# Parse Command-line Arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    -s|--slug) SLUG="$2"; shift 2 ;;
    -t|--tag) TAG="$2"; shift 2 ;;
    -d|--dir) DIST_DIR="$2"; shift 2 ;;
    -n|--notes) NOTES="$2"; shift 2 ;;
    --channel) CHANNEL="$2"; shift 2 ;;
    --notify) NOTIFY=true; shift ;;
    --dry-run) DRY_RUN=true; shift ;;
    --no-github) ENABLE_GITHUB=false; shift ;;
    --no-r2) ENABLE_R2=false; shift ;;
    --no-ypc) ENABLE_YPC=false; shift ;;
    --no-gcc) ENABLE_GCC=false; shift ;;
    --no-manifest) ENABLE_MANIFEST=false; shift ;;
    -j|--jobs) PARALLEL_JOBS="$2"; shift 2 ;;
    -r|--retries) MAX_RETRIES="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    -v|--version) echo "caf-distribute v$VERSION"; exit 0 ;;
    *) log_err "Unknown argument: $1"; usage; exit 1 ;;
  esac
done

# Try autodetection of slug if not provided
if [[ -z "$SLUG" ]]; then
  if [[ -f "app.toml" ]] && grep -q '^slug' "app.toml"; then
    SLUG=$(grep '^slug' "app.toml" | head -n1 | cut -d'=' -f2 | tr -d ' "' | tr -d "'")
    log_info "Auto-detected slug from app.toml: $SLUG"
  elif [[ -f "crates/caterm-app/tauri.conf.json" ]]; then
    SLUG="caterm"
    log_info "Auto-detected slug: caterm"
  elif [[ -f "crates/caf-app/tauri.conf.json" ]]; then
    SLUG="caframework"
    log_info "Auto-detected slug: caframework"
  fi
fi

if [[ -z "$SLUG" ]]; then
  log_err "Missing app slug. Specify with --slug <name> or set RELEASE_SLUG."
  exit 1
fi

if [[ -z "$TAG" ]]; then
  if git describe --tags --exact-match 2>/dev/null; then
    TAG=$(git describe --tags --exact-match)
    log_info "Auto-detected tag from git: $TAG"
  fi
fi

if [[ -z "$TAG" ]]; then
  log_err "Missing release tag. Specify with --tag <version> (e.g. v1.0.0) or set RELEASE_TAG."
  exit 1
fi

# Ensure tag starts with 'v' for standard release tagging
RAW_VERSION="${TAG#v}"

# Resolve Release Notes
RELEASE_NOTES_CONTENT=""
if [[ -n "$NOTES" ]]; then
  if [[ -f "$NOTES" ]]; then
    RELEASE_NOTES_CONTENT=$(cat "$NOTES")
  else
    RELEASE_NOTES_CONTENT="$NOTES"
  fi
elif command -v gh >/dev/null 2>&1 && [[ -n "$GITHUB_REPO" ]]; then
  RELEASE_NOTES_CONTENT=$(gh release view "$TAG" --repo "$GITHUB_REPO" --json body --jq .body 2>/dev/null || true)
fi

# If notes still empty, provide default
if [[ -z "$RELEASE_NOTES_CONTENT" ]]; then
  RELEASE_NOTES_CONTENT="Release $TAG for $SLUG"
fi

# Check required commands
require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    log_err "Required command not found in PATH: $1"
    return 1
  fi
  return 0
}

require_cmd sha256sum || exit 1
require_cmd jq || exit 1
require_cmd curl || exit 1

# Retry Helper with exponential backoff
run_with_retry() {
  local desc="$1"
  shift
  local attempt=1
  local delay=2

  while true; do
    if "$@"; then
      return 0
    else
      local exit_code=$?
      if [[ $attempt -ge $MAX_RETRIES ]]; then
        log_err "Failed '$desc' after $attempt attempts (exit code: $exit_code)."
        return $exit_code
      fi
      log_warn "Attempt $attempt/$MAX_RETRIES failed for '$desc'. Retrying in ${delay}s..."
      sleep "$delay"
      attempt=$((attempt + 1))
      delay=$((delay * 2))
    fi
  done
}

# ------------------------------------------------------------------------------
# 1. Scan and Validate Artifacts
# ------------------------------------------------------------------------------
log_step "Step 1: Inspecting Artifacts in '$DIST_DIR'"

if [[ ! -d "$DIST_DIR" ]]; then
  log_err "Artifact directory '$DIST_DIR' does not exist."
  exit 1
fi

ARTIFACT_FILES=()
SIG_FILES=()
ALL_FILES=()

# Collect supported release artifacts and signatures
for f in "$DIST_DIR"/*; do
  [[ -f "$f" ]] || continue
  filename=$(basename "$f")
  case "$filename" in
    *.exe|*.msi|*.deb|*.rpm|*.AppImage|*.apk|*.tar.gz|*.zip)
      ARTIFACT_FILES+=("$f")
      ALL_FILES+=("$f")
      ;;
    *.sig|*.idsig)
      SIG_FILES+=("$f")
      ALL_FILES+=("$f")
      ;;
    *)
      # Ignore other unrecognised files or temp files unless SHA256SUMS/latest.json
      ;;
  esac
done

if [[ ${#ARTIFACT_FILES[@]} -eq 0 && "$DRY_RUN" = false ]]; then
  log_err "No valid release artifacts (.exe, .apk, .deb, .rpm, .AppImage, etc.) found in '$DIST_DIR'."
  exit 1
fi

log_info "Discovered ${#ARTIFACT_FILES[@]} installer artifact(s) and ${#SIG_FILES[@]} signature file(s):"
for f in "${ARTIFACT_FILES[@]}"; do
  size_human=$(du -h "$f" | cut -f1)
  echo "  - [Installer] $(basename "$f") ($size_human)"
done
for f in "${SIG_FILES[@]}"; do
  echo "  - [Signature] $(basename "$f")"
done

# ------------------------------------------------------------------------------
# 2. Compute SHA-256 Checksums & Generate SHA256SUMS
# ------------------------------------------------------------------------------
log_step "Step 2: Computing SHA-256 Checksums"

SUMS_FILE="$DIST_DIR/SHA256SUMS"
CHECKSUMS_JSON="$DIST_DIR/checksums.json"
> "$SUMS_FILE"

declare -A ARTIFACT_SHAS
declare -A ARTIFACT_SIZES

for file in "${ALL_FILES[@]}"; do
  fname=$(basename "$file")
  sha=$(sha256sum "$file" | awk '{print $1}')
  size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file")
  ARTIFACT_SHAS["$fname"]="$sha"
  ARTIFACT_SIZES["$fname"]="$size"
  echo "$sha  $fname" >> "$SUMS_FILE"
  log_ok "$fname: $sha ($size bytes)"
done

ALL_FILES+=("$SUMS_FILE")

# ------------------------------------------------------------------------------
# 3. Generate Tauri v2 Auto-Updater Manifest (`latest.json`)
# ------------------------------------------------------------------------------
if [[ "$ENABLE_MANIFEST" = true ]]; then
  log_step "Step 3: Generating Tauri v2 Updater Manifest (latest.json)"

  MANIFEST_FILE="$DIST_DIR/latest.json"
  PUB_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

  # Find platform files & signatures
  WIN_INSTALLER=""
  WIN_SIG=""
  LINUX_INSTALLER=""
  LINUX_SIG=""
  DARWIN_INSTALLER=""
  DARWIN_SIG=""

  for f in "${ARTIFACT_FILES[@]}"; do
    fname=$(basename "$f")
    if [[ "$fname" =~ \.exe$ || "$fname" =~ \.msi$ ]]; then
      WIN_INSTALLER="$fname"
      if [[ -f "$DIST_DIR/${fname}.sig" ]]; then
        WIN_SIG=$(cat "$DIST_DIR/${fname}.sig" | tr -d '\r\n')
      fi
    elif [[ "$fname" =~ \.AppImage$ ]]; then
      LINUX_INSTALLER="$fname"
      if [[ -f "$DIST_DIR/${fname}.sig" ]]; then
        LINUX_SIG=$(cat "$DIST_DIR/${fname}.sig" | tr -d '\r\n')
      fi
    elif [[ "$fname" =~ \.dmg$ || "$fname" =~ \.app\.tar\.gz$ ]]; then
      DARWIN_INSTALLER="$fname"
      if [[ -f "$DIST_DIR/${fname}.sig" ]]; then
        DARWIN_SIG=$(cat "$DIST_DIR/${fname}.sig" | tr -d '\r\n')
      fi
    fi
  done

  # Prefer GitHub URL or GCC download proxy URL for manifest
  BASE_URL="https://github.com/${GITHUB_REPO:-cecep-azhar/$SLUG}/releases/download/${TAG}"
  if [[ -n "$GCC_BASE_URL" && "$ENABLE_GCC" = true ]]; then
    DL_BASE="${GCC_BASE_URL}/dl/${SLUG}/v/${TAG}"
  else
    DL_BASE="$BASE_URL"
  fi

  # Build platforms object dynamically
  PLATFORMS_OBJ="{}"

  if [[ -n "$WIN_INSTALLER" && -n "$WIN_SIG" ]]; then
    PLATFORMS_OBJ=$(echo "$PLATFORMS_OBJ" | jq \
      --arg sig "$WIN_SIG" \
      --arg url "${BASE_URL}/${WIN_INSTALLER}" \
      '. + {"windows-x86_64": {"signature": $sig, "url": $url}}')
  fi

  if [[ -n "$LINUX_INSTALLER" && -n "$LINUX_SIG" ]]; then
    PLATFORMS_OBJ=$(echo "$PLATFORMS_OBJ" | jq \
      --arg sig "$LINUX_SIG" \
      --arg url "${BASE_URL}/${LINUX_INSTALLER}" \
      '. + {"linux-x86_64": {"signature": $sig, "url": $url}}')
  fi

  if [[ -n "$DARWIN_INSTALLER" && -n "$DARWIN_SIG" ]]; then
    PLATFORMS_OBJ=$(echo "$PLATFORMS_OBJ" | jq \
      --arg sig "$DARWIN_SIG" \
      --arg url "${BASE_URL}/${DARWIN_INSTALLER}" \
      '. + {"darwin-x86_64": {"signature": $sig, "url": $url}, "darwin-aarch64": {"signature": $sig, "url": $url}}')
  fi

  NOTES_URL="https://github.com/${GITHUB_REPO:-cecep-azhar/$SLUG}/releases/tag/${TAG}"

  jq -n \
    --arg version "$RAW_VERSION" \
    --arg notes "$NOTES_URL" \
    --arg pub_date "$PUB_DATE" \
    --argjson platforms "$PLATFORMS_OBJ" \
    '{
      version: $version,
      notes: $notes,
      pub_date: $pub_date,
      platforms: $platforms
    }' > "$MANIFEST_FILE"

  log_ok "Generated $MANIFEST_FILE:"
  cat "$MANIFEST_FILE" | jq .
  ALL_FILES+=("$MANIFEST_FILE")
fi

# ------------------------------------------------------------------------------
# 4. Storage S3 Upload Helper (Native Python SigV4 / AWS CLI Fallback)
# ------------------------------------------------------------------------------
upload_s3() {
  local provider_name="$1"
  local endpoint="$2"
  local bucket="$3"
  local ak="$4"
  local sk="$5"
  local region="$6"
  local prefix="$7"
  local filepath="$8"

  local filename
  filename=$(basename "$filepath")
  local object_key="${prefix}${filename}"

  if [[ -z "$bucket" || -z "$ak" || -z "$sk" ]]; then
    log_warn "[$provider_name] Missing credentials or bucket name (Bucket: '$bucket', AK: '${ak:0:4}...'). Skipping."
    return 0
  fi

  if [[ "$DRY_RUN" = true ]]; then
    log_info "[$provider_name] [DRY RUN] Would upload '$filename' -> s3://$bucket/$object_key (Endpoint: $endpoint)"
    return 0
  fi

  log_info "[$provider_name] Uploading '$filename' to s3://$bucket/$object_key..."

  # Method 1: Check if AWS CLI is installed
  if command -v aws >/dev/null 2>&1; then
    local aws_args=(s3 cp "$filepath" "s3://$bucket/$object_key")
    if [[ -n "$endpoint" ]]; then
      aws_args+=(--endpoint-url "$endpoint")
    fi
    if [[ -n "$region" ]]; then
      aws_args+=(--region "$region")
    fi

    AWS_ACCESS_KEY_ID="$ak" AWS_SECRET_ACCESS_KEY="$sk" aws "${aws_args[@]}"
    return $?
  fi

  # Method 2: Check if rclone is installed
  if command -v rclone >/dev/null 2>&1; then
    RCLONE_CONFIG_TEMP_TYPE="s3" \
    RCLONE_CONFIG_TEMP_PROVIDER="Other" \
    RCLONE_CONFIG_TEMP_ACCESS_KEY_ID="$ak" \
    RCLONE_CONFIG_TEMP_SECRET_ACCESS_KEY="$sk" \
    RCLONE_CONFIG_TEMP_ENDPOINT="$endpoint" \
    RCLONE_CONFIG_TEMP_REGION="$region" \
    rclone copyto "$filepath" ":s3,endpoint='$endpoint',access_key_id='$ak',secret_access_key='$sk',region='$region':$bucket/$object_key"
    return $?
  fi

  # Method 3: Fallback to Universal Python S3 SigV4 Upload Script (no external pip dependencies)
  python3 - << PYEOF
import os, sys, hashlib, hmac, datetime, urllib.request, mimetypes

endpoint = "${endpoint}"
bucket = "${bucket}"
object_key = "${object_key}".lstrip('/')
access_key = "${ak}"
secret_key = "${sk}"
region = "${region}" if "${region}" else "us-east-1"
filepath = "${filepath}"

if not endpoint.startswith("http://") and not endpoint.startswith("https://"):
    endpoint = "https://" + endpoint

# Clean endpoint url
endpoint = endpoint.rstrip('/')

# S3 Path-style url
url = f"{endpoint}/{bucket}/{object_key}"

with open(filepath, 'rb') as f:
    payload = f.read()

payload_hash = hashlib.sha256(payload).hexdigest()
content_type, _ = mimetypes.guess_type(filepath)
if not content_type:
    content_type = "application/octet-stream"

t = datetime.datetime.utcnow()
amz_date = t.strftime('%Y%m%dT%H:%M:%SZ')
date_stamp = t.strftime('%Y%m%d')

# Parse host from endpoint
from urllib.parse import urlparse
parsed = urlparse(url)
host = parsed.netloc
canonical_uri = parsed.path

canonical_headers = f"content-type:{content_type}\nhost:{host}\nx-amz-content-sha256:{payload_hash}\nx-amz-date:{amz_date}\n"
signed_headers = "content-type;host;x-amz-content-sha256;x-amz-date"
canonical_request = f"PUT\n{canonical_uri}\n\n{canonical_headers}\n{signed_headers}\n{payload_hash}"

algorithm = 'AWS4-HMAC-SHA256'
credential_scope = f"{date_stamp}/{region}/s3/aws4_request"
string_to_sign = f"{algorithm}\n{amz_date}\n{credential_scope}\n{hashlib.sha256(canonical_request.encode('utf-8')).hexdigest()}"

def sign(key, msg):
    return hmac.new(key, msg.encode('utf-8'), hashlib.sha256).digest()

k_date = sign(('AWS4' + secret_key).encode('utf-8'), date_stamp)
k_region = sign(k_date, region)
k_service = sign(k_region, "s3")
k_signing = sign(k_service, "aws4_request")
signature = hmac.new(k_signing, string_to_sign.encode('utf-8'), hashlib.sha256).hexdigest()

authorization_header = f"{algorithm} Credential={access_key}/{credential_scope}, SignedHeaders={signed_headers}, Signature={signature}"

req = urllib.request.Request(url, data=payload, method="PUT")
req.add_header('Host', host)
req.add_header('Content-Type', content_type)
req.add_header('x-amz-date', amz_date)
req.add_header('x-amz-content-sha256', payload_hash)
req.add_header('Authorization', authorization_header)

try:
    with urllib.request.urlopen(req) as resp:
        if resp.status in (200, 201, 204):
            sys.exit(0)
        else:
            print(f"Upload failed with status {resp.status}", file=sys.stderr)
            sys.exit(1)
except Exception as e:
    print(f"Upload error: {e}", file=sys.stderr)
    sys.exit(1)
PYEOF
}

# ------------------------------------------------------------------------------
# 5. GCC Storage Asset Upload & Publish Helper
# ------------------------------------------------------------------------------
upload_gcc_asset() {
  local filepath="$1"
  local filename
  filename=$(basename "$filepath")

  if [[ -z "$GCC_API_KEY" ]]; then
    log_warn "[GCC API] GCC_API_KEY is not set. Skipping GCC Asset Upload."
    return 0
  fi

  local encoded_name
  encoded_name=$(jq -rn --arg n "$filename" '$n|@uri')
  local upload_url="${GCC_BASE_URL}/api/v1/releases/${SLUG}/versions/${TAG}/assets?name=${encoded_name}"

  if [[ "$DRY_RUN" = true ]]; then
    log_info "[GCC API] [DRY RUN] Would upload '$filename' -> $upload_url"
    return 0
  fi

  log_info "[GCC API] Uploading '$filename' to $upload_url..."

  local resp_code
  resp_code=$(curl -s -w "%{http_code}" -o /tmp/gcc_upload_resp.json \
    --fail-with-body --retry "$MAX_RETRIES" --retry-all-errors \
    -X POST "$upload_url" \
    -H "X-API-Key: ${GCC_API_KEY}" \
    -H "Content-Type: application/octet-stream" \
    --data-binary "@${filepath}") || {
      local err_content=$(cat /tmp/gcc_upload_resp.json 2>/dev/null || echo "Unknown error")
      log_err "[GCC API] Failed to upload $filename: HTTP $resp_code ($err_content)"
      return 1
    }

  log_ok "[GCC API] Uploaded $filename successfully (HTTP $resp_code)"
  return 0
}

publish_gcc_release() {
  if [[ -z "$GCC_API_KEY" ]]; then
    return 0
  fi

  local publish_url="${GCC_BASE_URL}/api/v1/releases/${SLUG}/versions/${TAG}/publish"

  if [[ "$DRY_RUN" = true ]]; then
    log_info "[GCC API] [DRY RUN] Would publish release $TAG with channel=$CHANNEL, notify=$NOTIFY"
    return 0
  fi

  log_info "[GCC API] Publishing release version $TAG on GCC..."

  local payload
  payload=$(jq -n \
    --arg notes "$RELEASE_NOTES_CONTENT" \
    --arg channel "$CHANNEL" \
    --argjson notify "$NOTIFY" \
    '{notes: $notes, channel: $channel, notify: $notify}')

  local resp_code
  resp_code=$(curl -s -w "%{http_code}" -o /tmp/gcc_publish_resp.json \
    --fail-with-body \
    -X POST "$publish_url" \
    -H "X-API-Key: ${GCC_API_KEY}" \
    -H "Content-Type: application/json" \
    --data-binary "$payload") || {
      local err_content=$(cat /tmp/gcc_publish_resp.json 2>/dev/null || echo "Unknown error")
      log_warn "[GCC API] Publish returned HTTP $resp_code: $err_content"
      return 0
    }

  log_ok "[GCC API] Published release $TAG successfully (HTTP $resp_code)"
  cat /tmp/gcc_publish_resp.json | jq . 2>/dev/null || true
  return 0
}

# ------------------------------------------------------------------------------
# 6. GitHub Release Upload Helper
# ------------------------------------------------------------------------------
upload_github() {
  if ! command -v gh >/dev/null 2>&1; then
    log_warn "[GitHub] 'gh' CLI tool not found. Skipping GitHub Release upload."
    return 0
  fi

  local gh_repo_flag=()
  if [[ -n "$GITHUB_REPO" ]]; then
    gh_repo_flag=(--repo "$GITHUB_REPO")
  fi

  if [[ "$DRY_RUN" = true ]]; then
    log_info "[GitHub] [DRY RUN] Would upload all files to GitHub release '$TAG' (${gh_repo_flag[*]:-current repo})"
    return 0
  fi

  log_info "[GitHub] Uploading artifacts to GitHub Release $TAG..."

  # Create release if it doesn't exist yet
  if ! gh release view "$TAG" "${gh_repo_flag[@]}" >/dev/null 2>&1; then
    log_info "[GitHub] Release $TAG does not exist. Creating release..."
    gh release create "$TAG" \
      "${gh_repo_flag[@]}" \
      --title "$SLUG $TAG" \
      --notes "$RELEASE_NOTES_CONTENT" \
      $([[ "$CHANNEL" == "beta" ]] && echo "--prerelease")
  fi

  # Upload all files
  gh release upload "$TAG" "${ALL_FILES[@]}" --clobber "${gh_repo_flag[@]}"
  log_ok "[GitHub] Uploaded ${#ALL_FILES[@]} file(s) to GitHub release $TAG."
}

# ------------------------------------------------------------------------------
# 7. Orchestration & Parallel Execution
# ------------------------------------------------------------------------------
log_step "Step 4: Executing Multi-Mirror Distribution"

log_info "Configuration Overview:"
echo "  - Slug:           $SLUG"
echo "  - Tag:            $TAG"
echo "  - Channel:        $CHANNEL"
echo "  - Dry Run:        $DRY_RUN"
echo "  - Parallel Jobs:  $PARALLEL_JOBS"
echo "  - Total Files:    ${#ALL_FILES[@]}"
echo "  - GitHub Target:  $([[ "$ENABLE_GITHUB" = true ]] && echo "ENABLED" || echo "DISABLED")"
echo "  - R2 Target:      $([[ "$ENABLE_R2" = true ]] && echo "ENABLED" || echo "DISABLED")"
echo "  - YPC S3 Target:  $([[ "$ENABLE_YPC" = true ]] && echo "ENABLED" || echo "DISABLED")"
echo "  - GCC API Target: $([[ "$ENABLE_GCC" = true ]] && echo "ENABLED" || echo "DISABLED")"

# Prepare Task Queue for Parallelism (xargs / background processes)
TASKS_FILE=$(mktemp /tmp/release_tasks_XXXXXX.txt)

# 1. Enqueue R2 Uploads
if [[ "$ENABLE_R2" = true && -n "$R2_BUCKET" ]]; then
  r2_p="${R2_PREFIX:-$SLUG/v/$TAG/}"
  for f in "${ALL_FILES[@]}"; do
    echo "upload_s3 'Cloudflare R2' '$R2_ENDPOINT' '$R2_BUCKET' '$R2_ACCESS_KEY_ID' '$R2_SECRET_ACCESS_KEY' '$R2_REGION' '$r2_p' '$f'" >> "$TASKS_FILE"
  done
fi

# 2. Enqueue YPC S3 Uploads
if [[ "$ENABLE_YPC" = true && -n "$YPC_BUCKET" ]]; then
  ypc_p="${YPC_PREFIX:-$SLUG/v/$TAG/}"
  for f in "${ALL_FILES[@]}"; do
    echo "upload_s3 'YPC S3' '$YPC_ENDPOINT' '$YPC_BUCKET' '$YPC_ACCESS_KEY_ID' '$YPC_SECRET_ACCESS_KEY' '$YPC_REGION' '$ypc_p' '$f'" >> "$TASKS_FILE"
  done
fi

# Export functions for subshells if needed
export -f upload_s3
export -f log_info log_ok log_warn log_err

# Execute S3 parallel uploads if tasks exist
TOTAL_S3_TASKS=$(wc -l < "$TASKS_FILE" || echo 0)
if [[ $TOTAL_S3_TASKS -gt 0 ]]; then
  log_info "Running $TOTAL_S3_TASKS S3 upload task(s) with concurrency of $PARALLEL_JOBS..."
  if command -v xargs >/dev/null 2>&1; then
    # Use bash subshell with xargs
    xargs -I {} -P "$PARALLEL_JOBS" bash -c "{}" < "$TASKS_FILE"
  else
    while IFS= read -r task; do
      eval "$task"
    done < "$TASKS_FILE"
  fi
fi
rm -f "$TASKS_FILE"

# 3. GCC Asset Uploads (Sequential / Installers first, then signatures as per GCC contract)
if [[ "$ENABLE_GCC" = true ]]; then
  log_step "Step 4.1: Uploading to GCC Release Storage API"
  # Installers first
  for f in "${ARTIFACT_FILES[@]}"; do
    run_with_retry "GCC upload $(basename "$f")" upload_gcc_asset "$f"
  done
  # Then signatures
  for f in "${SIG_FILES[@]}"; do
    run_with_retry "GCC upload signature $(basename "$f")" upload_gcc_asset "$f"
  done
  # SHA256SUMS & latest.json
  if [[ -f "$SUMS_FILE" ]]; then
    run_with_retry "GCC upload SHA256SUMS" upload_gcc_asset "$SUMS_FILE"
  fi
  if [[ "$ENABLE_MANIFEST" = true && -f "$MANIFEST_FILE" ]]; then
    run_with_retry "GCC upload latest.json" upload_gcc_asset "$MANIFEST_FILE"
  fi

  # Publish release on GCC
  publish_gcc_release
fi

# 4. GitHub Releases Upload
if [[ "$ENABLE_GITHUB" = true ]]; then
  log_step "Step 4.2: Uploading to GitHub Releases"
  run_with_retry "GitHub Release upload" upload_github
fi

# ------------------------------------------------------------------------------
# 8. Summary Report
# ------------------------------------------------------------------------------
log_step "Release Distribution Completed Successfully!"
echo "Summary:"
echo "  - Application:    $SLUG"
echo "  - Version / Tag:  $TAG"
echo "  - Checksums:      $SUMS_FILE"
if [[ "$ENABLE_MANIFEST" = true ]]; then
  echo "  - Updater JSON:   $MANIFEST_FILE"
fi
echo "  - Total Files:    ${#ALL_FILES[@]}"
echo "================================================================================"
