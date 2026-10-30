#!/usr/bin/env bash
# adr-index.sh - SessionStart hook: lists the project's Accepted ADRs, one line
# each, so every session knows which decisions hold before it changes code.
# Claude Code adds a SessionStart hook's stdout to the session's context.
#
#   Accepted ADRs in docs/adr/ (read one before changing what it decides):
#   0002 Layered code review: ... - docs/adr/0002-code-review-mechanism.md
#
# Accepted: the first non-blank line under `## Status` starts with `Accepted`,
# as the commit gate's ADR check reads it (docs/adr/0015). Proposed, Rejected,
# and fully superseded ADRs are left out: the list is what holds now.
#
# The project is $CLAUDE_PROJECT_DIR, else the git root of the working
# directory, else the working directory. Silent when there is no docs/adr/ or
# no Accepted ADR, and on any error. Bash and POSIX tools only.
set -u

root="${CLAUDE_PROJECT_DIR:-}"
[ -n "$root" ] || root="$(git rev-parse --show-toplevel 2>/dev/null)" || root="$PWD"
dir="$root/docs/adr"
[ -d "$dir" ] || exit 0

lines=""
for f in "$dir"/[0-9][0-9][0-9][0-9]-*.md; do
  [ -f "$f" ] || continue
  line="$(tr -d '\r' < "$f" 2>/dev/null | awk -v path="docs/adr/${f##*/}" '
    title == "" && /^# / { title = substr($0, 3); sub(/^[0-9]+\.[[:space:]]*/, "", title) }
    /^## / { if (in_status) exit; in_status = ($0 ~ /^## Status[[:space:]]*$/); next }
    in_status && NF { accepted = ($0 ~ /^Accepted/); exit }
    END { if (accepted && title != "") print title " - " path }')" || continue
  [ -n "$line" ] || continue
  num="${f##*/}"
  lines="${lines}${num%%-*} ${line}"$'\n'
done
[ -n "$lines" ] || exit 0

printf 'Accepted ADRs in docs/adr/ (read one before changing what it decides; supersede it rather than edit it):\n%s' "$lines"
exit 0
