#!/usr/bin/env bash
# T2-BOOT-06 budget gate. Reads bench-report.json (from scripts/bench.sh) and fails
# (exit != 0) if any Fase-0-measurable REQ-02 metric exceeds its threshold. Thresholds
# are overridable via CATERM_BUDGET_* env vars so the gate's bite can be proven on demand.
#
# MEMORY METRIC (direvisi 2026-09-18, koreksi kedua):
#   Gerbang ini memakai `private_bytes` — memori privat pohon proses, halaman bersama
#   TIDAK dihitung. Versi pertama harness ini memakai jumlah *working set* dan melaporkan
#   ~330 MB untuk aplikasi kosong; angka itu salah karena ~200 MB kode Chromium
#   (msedge.dll) yang dipakai bersama terhitung ulang di setiap proses WebView2.
#   `working_set_sum_bytes` tetap ada di laporan untuk transparansi, tapi DILARANG
#   dipakai sebagai angka budget. Lihat crates/caf-cli/src/bench.rs.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT="${1:-$PROJECT_ROOT/bench-report.json}"

if [ ! -f "$REPORT" ]; then
    echo "bench-report.json tidak ada di $REPORT - jalankan scripts/bench.sh dulu" >&2
    exit 2
fi

extract_num() {
    grep "\"$1\"" "$REPORT" | head -1 | sed -E 's/[^:]*:[[:space:]]*([0-9]+).*/\1/'
}

BINARY_SIZE_BYTES=$(extract_num binary_size_bytes)
COLD_START_MS=$(extract_num cold_start_ms)
PRIVATE_BYTES=$(extract_num private_bytes)

# private_bytes = null (platform belum didukung, mis. macOS) harus GAGAL dengan jelas,
# bukan lolos diam-diam karena string kosong dianggap 0.
if ! [[ "$PRIVATE_BYTES" =~ ^[0-9]+$ ]]; then
    echo "GAGAL: private_bytes tidak tersedia di laporan (null/tidak terparse)." >&2
    echo "  Platform ini belum punya akuntansi memori privat di bench.rs." >&2
    echo "  Gerbang menolak lolos tanpa angka - lihat bench.rs private_bytes_for()." >&2
    exit 3
fi

# Ambang memori direvisi 2026-09-18 berdasarkan pengukuran NYATA aplikasi kosong
# (Tauri 2 + WebView2, Windows): private_bytes = 155.783.168 (148,6 MiB), 7 proses.
# 148,6 x 1,15 headroom ~= 171 -> dibulatkan ke 180 MB. Silang-periksa manusia dengan
# private working set Task Manager pada run yang sama: ~70 MB (lihat evidence
# T2-BOOT-06b). Target lama 35 MB terbukti tidak realistis dengan WebView2 (Q-01/D-01);
# angka 380 MB dari revisi pertama juga SALAH dan sudah dicabut.
BUDGET_BINARY_MB="${CATERM_BUDGET_BINARY_MB:-8}"
BUDGET_COLD_START_MS="${CATERM_BUDGET_COLD_START_MS:-800}"
BUDGET_PRIVATE_MB="${CATERM_BUDGET_PRIVATE_MB:-160}"

# Round UP to the nearest whole MB - never let integer truncation hide an overage.
BINARY_SIZE_MB=$(((BINARY_SIZE_BYTES + 1048575) / 1048576))
PRIVATE_MB=$(((PRIVATE_BYTES + 1048575) / 1048576))

FAIL=0

check() {
    local name="$1" actual="$2" budget="$3" unit="$4"
    if [ "$actual" -gt "$budget" ]; then
        echo "GAGAL: $name = $actual $unit > ambang $budget $unit"
        FAIL=1
    else
        echo "OK: $name = $actual $unit <= ambang $budget $unit"
    fi
}

check "binary_size_mb" "$BINARY_SIZE_MB" "$BUDGET_BINARY_MB" "MB"
check "cold_start_ms" "$COLD_START_MS" "$BUDGET_COLD_START_MS" "ms"
check "private_mb (memori privat, empty-app proxy)" "$PRIVATE_MB" "$BUDGET_PRIVATE_MB" "MB"

echo "db_query_ms: belum tersedia - tabel hosts belum ada (Fase 1, T2-CORE-01), tidak dievaluasi"

exit $FAIL
