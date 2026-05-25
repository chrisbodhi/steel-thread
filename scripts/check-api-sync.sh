#!/usr/bin/env bash
# check-api-sync.sh
#
# Single source of truth for verifying that API-adjacent docs and the
# platerator-api Claude Skill are in sync with the OpenAPI-relevant
# source. Invoked by both the Claude PostToolUse hook (settings.json)
# and the GitHub Actions CI workflow (.github/workflows/ci.yml).
#
# Exit codes:
#   0  - hash matches; docs/skill believed to be in sync
#   1  - hash mismatch; remediation required
#   2  - misconfiguration (missing hash file, missing inputs)
#
# Failure output:
#   * GitHub Actions workflow command on stderr (picked up as PR annotation)
#   * Markdown remediation block appended to $GITHUB_STEP_SUMMARY when set
#     (renderable in `gh run view` and on the run summary page)
#   * Plain remediation text on stderr for local terminals and Claude hooks

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

HASH_FILE=".claude/skills/platerator-api/.api-hash"

# Files whose content drives the OpenAPI spec. Keep this list in sync
# with the schemas referenced from the ApiDoc struct in crates/web/src/lib.rs.
INPUTS=(
  "crates/web/src/lib.rs"
  "crates/domain/src/lib.rs"
)

for f in "${INPUTS[@]}"; do
  if [[ ! -f "$f" ]]; then
    echo "check-api-sync: missing input file: $f" >&2
    exit 2
  fi
done

compute_hash() {
  # sha256 each input, then hash the concatenated list so reordering
  # or renaming is detectable. Output is a single 64-char hex digest.
  sha256sum "${INPUTS[@]}" | sha256sum | awk '{print $1}'
}

CURRENT="$(compute_hash)"

if [[ "${1:-}" == "--write" ]]; then
  mkdir -p "$(dirname "$HASH_FILE")"
  printf '%s\n' "$CURRENT" > "$HASH_FILE"
  echo "check-api-sync: wrote $HASH_FILE ($CURRENT)"
  exit 0
fi

if [[ ! -f "$HASH_FILE" ]]; then
  echo "check-api-sync: hash file missing: $HASH_FILE" >&2
  echo "check-api-sync: run 'just update-api-hash' after reviewing the skill and docs." >&2
  exit 2
fi

EXPECTED="$(tr -d '[:space:]' < "$HASH_FILE")"

if [[ "$CURRENT" == "$EXPECTED" ]]; then
  exit 0
fi

# --- Mismatch: emit gh-friendly remediation ---------------------------------

REMEDIATION_TITLE="Platerator API out of sync with skill/docs"
REMEDIATION_BODY=$(cat <<EOF
The OpenAPI-relevant source changed but the committed hash in \`$HASH_FILE\` was not updated.

**Remediation (do all three, then refresh the hash):**

1. Update \`.claude/skills/platerator-api/SKILL.md\` to reflect the new endpoints, request bodies, or response schemas.
2. Update the endpoint table and \`## REST API\` section in \`CLAUDE.md\`.
3. Update the \`### Calling the API\` example fetch in \`CLAUDE.md\` if request/response shapes changed.

Then refresh the hash:

\`\`\`sh
just update-api-hash
git add $HASH_FILE .claude/skills/platerator-api/SKILL.md CLAUDE.md
\`\`\`

**Hashed inputs:** ${INPUTS[*]}
**Expected:** \`$EXPECTED\`
**Actual:**   \`$CURRENT\`
EOF
)

# 1. GitHub Actions annotation (file-level, shows on the PR Files tab).
#    The message must be a single line with %0A for newlines.
if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  ANNOT_MSG="OpenAPI-relevant source changed but .claude/skills/platerator-api/.api-hash was not updated.%0A%0ARun 'just update-api-hash' after updating SKILL.md and CLAUDE.md, then commit the hash file."
  printf '::error file=%s,title=%s::%s\n' \
    "crates/web/src/lib.rs" \
    "$REMEDIATION_TITLE" \
    "$ANNOT_MSG" >&2
fi

# 2. Step summary (markdown, renders on the run summary page and is
#    scrapeable via `gh run view --json`).
if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
  {
    echo "## ❌ $REMEDIATION_TITLE"
    echo
    echo "$REMEDIATION_BODY"
  } >> "$GITHUB_STEP_SUMMARY"
fi

# 3. Plain human-readable output (Claude hook / local terminal).
{
  echo "========================================================================"
  echo "check-api-sync: $REMEDIATION_TITLE"
  echo "------------------------------------------------------------------------"
  echo "$REMEDIATION_BODY"
  echo "========================================================================"
} >&2

exit 1
