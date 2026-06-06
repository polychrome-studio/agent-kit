#!/usr/bin/env bash
# Knowledge-layer — code-map refresh hook (PostToolUse on Bash).
#
# On every `git commit` that advances HEAD AND touches source files, regenerates
# knowledge/wiki/_codemap.md + knowledge/_codemap.json in the background.
#
# Discipline:
#   - HEAD-comparison guard: doc-only or failed commits never trigger.
#   - Source-file guard: skips if no .rs/.ts/.tsx/.js/.php/.py files changed.
#   - Background + time-boxed: never blocks the commit; uses `timeout` if available.
#   - Silent on any error: swallows all output; a broken codemap never fails a commit.
#   - state file: knowledge/journal/.last-codemap tracks the last regenerated HEAD.
#
# Output contract: none. This hook emits no PostToolUse JSON — breadcrumb hook
# already notified the agent about the commit. This one silently regenerates.
#
# Dependencies: uv (https://github.com/astral-sh/uv) in PATH; tree-sitter +
# tree-sitter-language-pack fetched on demand by uv (no permanent install needed).

set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$PWD}"
cd "$root" 2>/dev/null || exit 0
[ -d knowledge/journal ] || exit 0       # only act where a knowledge layer exists

input=$(cat)
cmd=$(printf '%s' "$input" | jq -r '.tool_input.command // ""' 2>/dev/null)
printf '%s' "$cmd" | grep -q 'git commit' || exit 0   # only git commit calls

head=$(git rev-parse --short HEAD 2>/dev/null) || exit 0

# Skip if HEAD didn't actually advance (failed/blocked commit)
state_crumb="knowledge/journal/.last-breadcrumb"
[ "$head" = "$(cat "$state_crumb" 2>/dev/null || true)" ] && exit 0

# Skip if no source files changed — doc-only commits don't need a remap
changed=$(git show --pretty=format: --name-only HEAD 2>/dev/null | sed '/^$/d')
if ! printf '%s\n' "$changed" | grep -qE '\.(rs|ts|tsx|js|mjs|cjs|php|py)$'; then
  exit 0
fi

# Locate codemap.py — fallback resolution chain (machine-portable):
#   1. Repo root's .claude/scripts/codemap.py  (in-repo copy, preferred)
#   2. Installed skill:  $HOME/.claude/skills/knowledge-layer/scripts/codemap.py
# If neither exists, exit 0 silently — never fail the commit.
repo_root="$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0
codemap_py=""
_candidate1="$repo_root/.claude/scripts/codemap.py"
_candidate2="$HOME/.claude/skills/knowledge-layer/scripts/codemap.py"
if   [ -f "$_candidate1" ]; then codemap_py="$_candidate1"
elif [ -f "$_candidate2" ]; then codemap_py="$_candidate2"
else exit 0   # neither path available — skip silently
fi

# Skip if uv not in PATH (portability: silent, not fatal)
command -v uv >/dev/null 2>&1 || exit 0

# Run in background, time-boxed. Use `timeout` if available, else just background.
regen() {
  uv run --quiet \
    --with tree-sitter \
    --with tree-sitter-language-pack \
    python3 "$codemap_py" "$root" \
    >/dev/null 2>&1
}

if command -v timeout >/dev/null 2>&1; then
  ( timeout 30 bash -c "$(declare -f regen); regen" ) &
else
  ( regen ) &
fi

disown 2>/dev/null || true
exit 0
