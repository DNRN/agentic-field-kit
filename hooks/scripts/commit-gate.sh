#!/usr/bin/env bash
# commit-gate.sh - PreToolUse gate that nudges code review AND documentation
# checks before `git commit`. One gate, one combined ask, independent waivers.
#
# Modes:
#   (stdin JSON)      gate mode: decide whether the attempted Bash command
#                     (a git commit) may proceed. Denies with a reason listing
#                     the pending check(s) and telling the agent to ask the
#                     user before committing.
#   --stamp-review [--all] [dir]  record the reviewed diff in .git/review-stamp
#   --stamp-docs   [--all] [dir]  record the documentation-checked code diff in
#                                 .git/doc-stamp
#   --all hashes the worktree diff (`git diff HEAD`), matching `git commit -a`;
#   the default hashes the staged diff.
#
# Checks:
#   review  pending when the diff is non-empty and its hash (full or code-only)
#           is not stamped. Code-only hash: doc edits after a review do not
#           invalidate it; any code change does. Doc-only diffs have no
#           code-only hash, so each one is reviewed on its own.
#   docs    pending when the diff changes code without touching any doc, the
#           repo tracks at least one doc, and the code-diff hash is not
#           stamped. A commit that edits docs is evidence docs were considered.
#
# What counts as a doc: git pathspecs in .commit-gate-docs at the repository
# root, one per line (`docs/`, `README.md`, `*.rst`, long-form magic such as
# `:(glob)docs/**`); blank lines and `#` comments ignored. Without the file:
# `*.md`. Everything else is code. See docs/adr/0004.
#
# Commit detection lives in parse-commit.py (chains, cd, git -C, git global
# options, combined short flags, sh -c, heredocs).
#
# Design: fail-open. Any unexpected state (no stdin, no python3, not a repo,
# parse error, git error) allows the commit. This is a nudge, not enforcement:
# `--no-verify` always bypasses. See docs/adr/0002, 0003, and 0004.
set -u

SCRIPT_PATH="$0"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

hash_of() { { sha256sum 2>/dev/null || shasum -a 256; } | cut -d' ' -f1; }

# load_docs <toplevel>: set DOCS (pathspecs that count as docs) and CODE_ONLY
# (diff arguments excluding them) from <toplevel>/.commit-gate-docs. Stamping
# and checking both call this, so their code-only hashes always agree.
load_docs() {
  local line
  DOCS=()
  if [ -f "$1/.commit-gate-docs" ]; then
    while IFS= read -r line || [ -n "$line" ]; do
      line="${line%$'\r'}"
      line="${line#"${line%%[![:space:]]*}"}"
      line="${line%"${line##*[![:space:]]}"}"
      case "$line" in ''|'#'*) continue ;; esac
      DOCS+=("$line")
    done < "$1/.commit-gate-docs"
  fi
  [ "${#DOCS[@]}" -gt 0 ] || DOCS=('*.md')
  CODE_ONLY=(-- .)
  for line in "${DOCS[@]}"; do
    case "$line" in
      ':('*) CODE_ONLY+=(":(exclude,${line#:(}") ;;
      *) CODE_ONLY+=(":(exclude)$line") ;;
    esac
  done
}

# diff_hashes <toplevel> <diff-kind>: print the full-diff hash, then the
# code-only hash when the code-only diff is non-empty. An empty code-only diff
# hashes to one constant shared by every doc-only change; stamping it would
# approve all future doc-only diffs.
diff_hashes() {
  local top="$1" kind="$2" rc
  git -C "$top" diff "$kind" 2>/dev/null | hash_of
  git -C "$top" diff "$kind" --quiet "${CODE_ONLY[@]}" 2>/dev/null
  rc=$?
  if [ "$rc" -eq 1 ]; then
    git -C "$top" diff "$kind" "${CODE_ONLY[@]}" 2>/dev/null | hash_of
  fi
}

stamp_mode() { # $1=flag $2=stamp-name, remaining: [--all] [dir]
  local flag="$1" name="$2" kind=--cached dir="$PWD" top git_dir
  shift 2
  if [ "${1:-}" = --all ]; then kind=HEAD; flag="$flag --all"; shift; fi
  [ $# -gt 0 ] && dir="$1"
  top="$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null)" || exit 0
  git_dir="$(git -C "$top" rev-parse --absolute-git-dir 2>/dev/null)" || exit 0
  load_docs "$top"
  diff_hashes "$top" "$kind" > "$git_dir/$name"
  echo "stamped $flag in $git_dir/$name"
  exit 0
}

case "${1:-}" in
  --stamp-review) shift; stamp_mode --stamp-review review-stamp "$@" ;;
  --stamp-docs)   shift; stamp_mode --stamp-docs   doc-stamp    "$@" ;;
esac

# --- gate mode --------------------------------------------------------------
input="$(cat 2>/dev/null)" || exit 0
[ -n "$input" ] || exit 0
# JSON and command parsing via python3 only (single code path). No python3 ->
# the gate cannot run -> fail open; a missing parser must not wedge commits.
command -v python3 >/dev/null 2>&1 || exit 0

# Only gate commit commands; everything else (and explicit bypasses) passes.
parsed="$(printf '%s' "$input" | python3 "$SCRIPT_DIR/parse-commit.py" 2>/dev/null)" || exit 0
{ IFS= read -r dir && IFS= read -r diff_kind; } <<EOF || exit 0
$parsed
EOF

top="$(git -C "$dir" rev-parse --show-toplevel 2>/dev/null)" || exit 0
git_dir="$(git -C "$top" rev-parse --absolute-git-dir 2>/dev/null)" || exit 0
load_docs "$top"

# Empty diff (allow-empty commits, merge continuations, nothing staged): allow.
# Any git error (e.g. `commit -a` before the first commit): fail open.
git -C "$top" diff "$diff_kind" --quiet 2>/dev/null
[ $? -eq 1 ] || exit 0

{ IFS= read -r full_hash; IFS= read -r code_hash || code_hash=""; } <<EOF
$(diff_hashes "$top" "$diff_kind")
EOF
[ "${#full_hash}" -ge 8 ] || exit 0  # hash failed -> fail open

docs_touched=no
if [ -n "$(git -C "$top" diff "$diff_kind" --name-only -- "${DOCS[@]}" 2>/dev/null)" ]; then
  docs_touched=yes
fi

stamped() { [ -f "$git_dir/$1" ] && grep -qxF -e "$2" "$git_dir/$1" 2>/dev/null; }

# --- review check -----------------------------------------------------------
review_pending=yes
if stamped review-stamp "$full_hash" || { [ -n "$code_hash" ] && stamped review-stamp "$code_hash"; }; then
  review_pending=no
fi

# --- docs check -------------------------------------------------------------
docs_pending=no
if [ -n "$code_hash" ] && [ "$docs_touched" = no ]; then
  if git -C "$top" ls-files -- "${DOCS[@]}" 2>/dev/null | grep -q .; then
    docs_pending=yes
    stamped doc-stamp "$code_hash" && docs_pending=no
  fi
fi

[ "$review_pending" = yes ] || [ "$docs_pending" = yes ] || exit 0

# --- combined denial --------------------------------------------------------
if [ "$diff_kind" = HEAD ]; then
  target=" worktree"; stamp_all=" --all"; scope="git commit -a: staged and unstaged changes"
else
  target=""; stamp_all=""; scope="staged changes"
fi
script_q="$(printf '%q' "$SCRIPT_PATH")"
top_q="$(printf '%q' "$top")"

pending=""
[ "$review_pending" = yes ] && pending="${pending}code review (unreviewed changes); "
[ "$docs_pending" = yes ] && pending="${pending}documentation (code changes with no documentation updates); "
pending="${pending%; }"

actions=""
[ "$review_pending" = yes ] && actions="${actions}run /agentic-toolkit:code-review${target}, then stamp: bash $script_q --stamp-review${stamp_all} $top_q. "
[ "$docs_pending" = yes ] && actions="${actions}run /agentic-toolkit:doc-sync${target}, then stamp: bash $script_q --stamp-docs${stamp_all} $top_q. "

reason="Pending before commit ($scope): $pending. Ask the user: 'Run code review / documentation check before committing?' If agreed: $actions For waived checks, rerun the commit with --no-verify."
python3 -c '
import json, sys
print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
    "permissionDecision": "deny", "permissionDecisionReason": sys.argv[1]}}))' "$reason" 2>/dev/null
exit 0
