#!/usr/bin/env bash
# Amber — journal breadcrumb hook (PostToolUse on Bash).
#
# When a `git commit` actually advances HEAD, append a crash-proof breadcrumb
# (time, short hash, subject, changed files) to today's journal entry, then
# nudge the agent to write the WHY while it's fresh. The commit is our
# "larger move" marker (see knowledge/CLAUDE.md, Rule 1). HEAD-comparison means
# failed/blocked commits (e.g. gitleaks) never breadcrumb, and we never double-write.
#
# Output contract: emit PostToolUse additionalContext JSON to reach the agent.
# Any non-fatal problem just exits 0 silently — journaling must never block work.

set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$PWD}"
cd "$root" 2>/dev/null || exit 0
[ -d knowledge/journal ] || exit 0          # only act inside the Amber repo

input=$(cat)
cmd=$(printf '%s' "$input" | jq -r '.tool_input.command // ""' 2>/dev/null)
printf '%s' "$cmd" | grep -q 'git commit' || exit 0   # only git commit calls

head=$(git rev-parse --short HEAD 2>/dev/null) || exit 0
state="knowledge/journal/.last-breadcrumb"
[ "$head" = "$(cat "$state" 2>/dev/null || true)" ] && exit 0   # no new commit — skip

date=$(date +%F)
time=$(date +%H:%M)
subject=$(git log -1 --pretty=%s 2>/dev/null)
# Clean ", "-joined file list, capped so big commits don't bloat the breadcrumb.
files=$(git show --pretty=format: --name-only HEAD 2>/dev/null | sed '/^$/d' \
  | awk 'NR<=12{printf "%s%s", (NR>1?", ":""), $0} END{if (NR>12) printf ", +%d more", NR-12}')

# Append into today's journal entry: the newest knowledge/journal/<today>*.md,
# or create a dated default if a session entry hasn't been started yet.
journal=$(ls -t knowledge/journal/${date}*.md 2>/dev/null | head -1)
if [ -z "$journal" ]; then
  journal="knowledge/journal/${date}-session.md"
  {
    printf -- '---\n'
    printf 'date: %s\n' "$date"
    printf 'session: session\n'
    printf 'status: in-progress\n'
    printf -- '---\n\n'
    printf '# Session — %s\n\n' "$date"
    printf '> Auto-breadcrumbs from commits below. Flesh out the **why** under each while fresh.\n'
  } > "$journal"
fi

printf '\n### %s — %s\n%s\nfiles: %s\n' "$time" "$head" "$subject" "${files:-—}" >> "$journal"
printf '%s' "$head" > "$state"

note="📓 Committed ${head} (${subject}). A breadcrumb was auto-appended to ${journal}. Add the WHY now while it's fresh — rationale, what was tried/abandoned, open threads — don't defer to session end (Rule 1, knowledge/CLAUDE.md)."
jq -nc --arg n "$note" \
  '{hookSpecificOutput:{hookEventName:"PostToolUse",additionalContext:$n}}'
exit 0
