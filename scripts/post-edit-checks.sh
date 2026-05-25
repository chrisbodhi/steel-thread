#!/usr/bin/env bash
# post-edit-checks.sh
#
# Non-blocking per-edit checks dispatched by file path. Invoked by the
# Claude PostToolUse hook (see .claude/settings.json); reads the hook's
# JSON payload on stdin and routes to clippy or typecheck based on the
# edited file.
#
# Design notes:
#   * Always exits 0 so a check failure never rolls back the edit.
#   * Failures are still printed prominently on stderr so Claude sees them
#     and surfaces the remediation inline.
#   * If GITHUB_ACTIONS is set, also emits ::warning:: annotations — this
#     script is currently only invoked from the hook, but the format is
#     CI-ready for a future fast-checks job.
#
# Matched edits:
#   crates/<crate>/src/**.rs   -> cargo clippy -p <crate> -- -D warnings
#   frontend/src/**.{ts,tsx}   -> (cd frontend && bun run typecheck)

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

INPUT=""
if [[ ! -t 0 ]]; then
  INPUT="$(cat)"
fi

# Extract tool_input.file_path without requiring jq. The hook payload is
# JSON with a predictable shape; this regex is sufficient for a single
# file_path field and skips if absent.
FILE_PATH=""
if [[ -n "$INPUT" ]]; then
  FILE_PATH=$(printf '%s' "$INPUT" \
    | grep -oE '"file_path"[[:space:]]*:[[:space:]]*"[^"]*"' \
    | head -1 \
    | sed -E 's/.*"file_path"[[:space:]]*:[[:space:]]*"([^"]*)".*/\1/')
fi

[[ -z "$FILE_PATH" ]] && exit 0

# Normalize to repo-relative path.
REL="${FILE_PATH#"$ROOT"/}"

run_and_warn() {
  local label="$1"; shift
  local output rc=0
  output="$("$@" 2>&1)" || rc=$?
  if (( rc != 0 )); then
    {
      echo "========================================================================"
      echo "post-edit-checks: $label failed (non-blocking, edit kept)"
      echo "  file: $REL"
      echo "  cmd:  $*"
      echo "------------------------------------------------------------------------"
      echo "$output"
      echo "========================================================================"
    } >&2
    if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
      printf '::warning file=%s,title=%s::%s\n' \
        "$REL" "post-edit-checks: $label failed" \
        "Run '$*' locally to reproduce. See step log for full output." >&2
    fi
  fi
}

if [[ "$REL" =~ ^crates/([^/]+)/src/.*\.rs$ ]]; then
  CRATE="${BASH_REMATCH[1]}"
  run_and_warn "clippy -p $CRATE" \
    cargo clippy -p "$CRATE" --quiet --no-deps -- -D warnings
elif [[ "$REL" =~ ^frontend/src/.*\.(ts|tsx)$ ]]; then
  run_and_warn "bun typecheck" \
    bash -c 'cd frontend && bun run typecheck'
fi

exit 0
