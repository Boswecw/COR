#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(pwd)"
GIT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"

if [[ -z "${GIT_ROOT}" ]]; then
  echo "ERROR: run this from inside the repo worktree so git can resolve the top-level root." >&2
  exit 1
fi

STAMP="$(date +%Y%m%d_%H%M%S)"
OUT_ROOT="${PROJECT_ROOT}/.tmp/centipede_queue_capture_${STAMP}"
PAYLOAD_ROOT="${OUT_ROOT}/payload"
SOURCE_ROOT="${PAYLOAD_ROOT}/repo_snapshot"
LOG_ROOT="${PAYLOAD_ROOT}/logs"
ZIP_PATH="${OUT_ROOT}/centipede_queue_capture_${STAMP}.zip"

TARGET_FILES=(
  "src/centipede_queue_claim.rs"
  "src/centipede_queue_reclaim.rs"
  "src/centipede_queue_complete.rs"
  "src/centipede_queue_heartbeat.rs"
  "src/centipede_queue_fail.rs"
  "src/bin/centipede_queue_reclaim.rs"
  "src/bin/centipede_queue_reclaim_smoke.rs"
  "src/bin/centipede_queue_complete_smoke.rs"
  "src/bin/centipede_queue_heartbeat.rs"
  "src/bin/centipede_queue_heartbeat_smoke.rs"
  "src/bin/centipede_queue_fail_smoke.rs"
)

SMOKE_BINS=(
  "centipede_queue_reclaim_smoke"
  "centipede_queue_complete_smoke"
  "centipede_queue_heartbeat_smoke"
  "centipede_queue_fail_smoke"
)

mkdir -p "$SOURCE_ROOT" "$LOG_ROOT"

{
  echo "project_root=${PROJECT_ROOT}"
  echo "git_root=${GIT_ROOT}"
  echo "captured_at=$(date --iso-8601=seconds)"
  echo "branch=$(git -C "$PROJECT_ROOT" rev-parse --abbrev-ref HEAD)"
  echo "head=$(git -C "$PROJECT_ROOT" rev-parse HEAD)"
} > "$LOG_ROOT/capture_meta.txt"

git -C "$PROJECT_ROOT" status --short > "$LOG_ROOT/git_status_short.txt"
git -C "$PROJECT_ROOT" status > "$LOG_ROOT/git_status_full.txt"
git -C "$PROJECT_ROOT" diff -- "${TARGET_FILES[@]}" > "$LOG_ROOT/git_diff_target_files.patch" || true

for path in "${TARGET_FILES[@]}"; do
  if [[ -f "$PROJECT_ROOT/$path" ]]; then
    mkdir -p "$SOURCE_ROOT/$(dirname "$path")"
    cp "$PROJECT_ROOT/$path" "$SOURCE_ROOT/$path"
  else
    echo "MISSING $path" >> "$LOG_ROOT/missing_files.txt"
  fi
done

cargo metadata --no-deps --format-version 1 > "$LOG_ROOT/cargo_metadata.json"

for bin in "${SMOKE_BINS[@]}"; do
  log_file="$LOG_ROOT/${bin}.txt"
  if cargo run --quiet --bin "$bin" > "$log_file" 2>&1; then
    printf 'PASS %s\n' "$bin" >> "$LOG_ROOT/smoke_summary.txt"
  else
    printf 'FAIL %s\n' "$bin" >> "$LOG_ROOT/smoke_summary.txt"
  fi
done

python3 - "$PAYLOAD_ROOT" "$ZIP_PATH" <<'PY'
import os
import sys
import zipfile

payload_root = sys.argv[1]
zip_path = sys.argv[2]

with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
    for root, _, files in os.walk(payload_root):
        for name in files:
            full_path = os.path.join(root, name)
            rel_path = os.path.relpath(full_path, payload_root)
            zf.write(full_path, rel_path)
PY

cat <<MSG
Capture complete.

Output directory:
  $OUT_ROOT

Capture zip:
  $ZIP_PATH

Important files:
  $LOG_ROOT/git_status_short.txt
  $LOG_ROOT/smoke_summary.txt
  $LOG_ROOT/git_diff_target_files.patch
MSG
