#!/usr/bin/env bash
# Regression tests for hooks/scripts/commit-gate.sh (PreToolUse contract:
# JSON on stdin, deny JSON on stdout or silent exit 0).
set -u
GATE="$(cd "$(dirname "$0")/.." && pwd)/hooks/scripts/commit-gate.sh"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
fails=0

# Sanitized env: interactive shells may shadow tools with divergent shims.
run_gate() { # $1=command $2=cwd -> prints the hook's stdout
  python3 -c 'import json,sys; print(json.dumps({"tool_input":{"command":sys.argv[1]},"cwd":sys.argv[2]}))' "$1" "$2" |
    env -i PATH=/usr/bin:/bin HOME="$WORK" bash "$GATE"
}
decision() { case "$(run_gate "$1" "$2")" in *'"deny"'*) echo deny ;; *) echo allow ;; esac; }
expect() { # $1=expected $2=description $3=command [$4=cwd]
  local got; got="$(decision "$3" "${4:-$REPO}")"
  if [ "$got" = "$1" ]; then echo "ok   $2"; else echo "FAIL $2: expected $1, got $got"; fails=$((fails + 1)); fi
}
expect_reason() { # $1=substring $2=description $3=command [$4=cwd]
  case "$(run_gate "$3" "${4:-$REPO}")" in
    *"$1"*) echo "ok   $2" ;;
    *) echo "FAIL $2: reason lacks '$1'"; fails=$((fails + 1)) ;;
  esac
}
expect_no_reason() {
  case "$(run_gate "$3" "${4:-$REPO}")" in
    *"$1"*) echo "FAIL $2: reason contains '$1'"; fails=$((fails + 1)) ;;
    *) echo "ok   $2" ;;
  esac
}
stamp() { env -i PATH=/usr/bin:/bin bash "$GATE" "$@" >/dev/null; }
new_repo() {
  REPO="$WORK/repo$RANDOM$RANDOM"
  git init -q "$REPO"
  git -C "$REPO" config user.email t@t; git -C "$REPO" config user.name t
  mkdir "$REPO/sub"
  echo a > "$REPO/code.py"; echo a > "$REPO/sub/lib.py"; echo d > "$REPO/README.md"
  git -C "$REPO" add -A; git -C "$REPO" commit -qm init
}
change() { echo "$RANDOM$RANDOM" >> "$REPO/$1"; }
stage() { change "$1"; git -C "$REPO" add "$1"; }

echo "# review check"
new_repo; stage code.py; stage README.md
expect deny "unreviewed staged change is denied" "git commit -m x"
expect_reason "--stamp-review" "denial names the stamp command" "git commit -m x"
stamp --stamp-review "$REPO"
expect allow "stamped diff passes" "git commit -m x"
stage README.md
expect allow "doc edit after review keeps the review" "git commit -m x"
stage code.py
expect deny "code change after review is denied" "git commit -m x"

echo "# doc-only diffs are each reviewed"
new_repo; stage README.md
stamp --stamp-review "$REPO"
git -C "$REPO" commit -qm docs --no-verify
stage README.md
expect deny "second unreviewed doc-only diff is denied" "git commit -m y"

echo "# commit -a"
new_repo; change code.py; change README.md
expect deny "-am with nothing staged is denied" "git commit -am wip"
expect deny "-a -m is denied" "git commit -a -m wip"
expect deny "--all is denied" "git commit --all -m wip"
expect_reason "--stamp-review --all" "denial names the --all stamp" "git commit -am wip"
stamp --stamp-review --all "$REPO"
expect allow "--all stamp unblocks -a commit" "git commit -am wip"
expect allow "nothing staged, no -a: empty diff" "git commit -m wip"

echo "# bypass only via real flags"
new_repo; stage code.py; stage README.md
expect deny "-n inside message is not a bypass" "git commit -m 'add -n flag'"
expect deny "--no-verify inside message is not a bypass" "git commit -m 'mention --no-verify'"
expect deny "-m value starting with n is not a bypass" "git commit -mno"
expect allow "-n bypasses" "git commit -n -m x"
expect allow "--no-verify bypasses" "git commit --no-verify -m x"
expect allow "combined -nm bypasses" "git commit -nm x"
expect allow "abbreviated --no-veri bypasses" "git commit --no-veri -m x"
expect deny "--verify after --no-verify re-enables" "git commit --no-verify --verify -m x"
expect deny "heredoc message with apostrophe and -n" "git commit -F - <<'EOF'
Don't skip
-n --no-verify
EOF"
expect deny "command-substitution heredoc message" "git commit -m \"\$(cat <<'EOF'
it's -n here
EOF
)\""

echo "# command forms"
expect deny "git -c k=v commit" "git -c core.x=1 commit -m z"
expect deny "git --no-pager commit" "git --no-pager commit -m z"
expect deny "git -C repo commit from elsewhere" "git -C '$REPO' commit -m z" "$WORK"
expect deny "cd repo && git commit" "cd '$REPO' && git commit -m z" "$WORK"
expect deny "env assignment prefix" "GIT_AUTHOR_NAME=x git commit -m z"
expect deny "sh -c wrapper" "sh -c 'git commit -m z'"
expect deny "second command in a chain" "git status && git commit -m z"
expect deny "commit from a subdirectory" "git commit -m z" "$REPO/sub"
stamp --stamp-review "$REPO/sub"
expect allow "stamp from a subdirectory covers the whole repo" "git commit -m z" "$REPO/sub"

echo "# fail-open and non-commits"
new_repo; stage code.py
expect allow "non-commit command" "git status"
expect allow "commit text in an echo" "echo 'git commit -m x'"
expect allow "git commit-tree is not commit" "git commit-tree HEAD^{tree}"
expect allow "non-repo cwd" "git commit -m x" "$WORK"
expect allow "unbalanced quotes" "git commit -m 'oops"
for payload in "" "not json" '{"tool_input":{}}'; do
  out="$(printf '%s' "$payload" | env -i PATH=/usr/bin:/bin bash "$GATE")"
  if [ -z "$out" ]; then echo "ok   malformed stdin '$payload' allowed"; else echo "FAIL malformed stdin '$payload'"; fails=$((fails + 1)); fi
done

echo "# docs check"
new_repo; stage code.py
expect_reason "documentation (" "code without docs raises the docs check" "git commit -m x"
stamp --stamp-docs "$REPO"
expect_no_reason "documentation (" "docs stamp clears the docs check" "git commit -m x"
stage README.md
expect_no_reason "documentation (" "doc edit in the commit satisfies the docs check" "git commit -m x"

echo "# docs paths from .commit-gate-docs"
# configure <content>: commit a .commit-gate-docs plus docs/guide.md,
# docs/a/deep.md, skills/x/SKILL.md, and guide.rst in a fresh repo.
configure() {
  new_repo
  mkdir -p "$REPO/docs/a" "$REPO/skills/x"
  for f in docs/guide.md docs/a/deep.md skills/x/SKILL.md guide.rst; do echo a > "$REPO/$f"; done
  printf '%b' "$1" > "$REPO/.commit-gate-docs"
  git -C "$REPO" add -A; git -C "$REPO" commit -qm config
}
configure '# docs live here\r\n\n  docs/  \r\nREADME.md'
stage skills/x/SKILL.md
expect_reason "documentation (" "markdown outside docs paths is code (comments, blanks, CRLF, padding ignored)" "git commit -m x"
stage docs/guide.md
expect_no_reason "documentation (" "edit under a configured docs path satisfies the docs check" "git commit -m x"
configure 'docs/\nREADME.md'
stage code.py; stage README.md
expect_no_reason "documentation (" "configured single file counts as docs" "git commit -m x"
stamp --stamp-review "$REPO"
stage docs/guide.md
expect allow "configured doc edit after review keeps the review" "git commit -m x"
stage skills/x/SKILL.md
expect_reason "code review" "markdown outside docs paths after review needs review" "git commit -m x"
configure 'docs/'
stage skills/x/SKILL.md
stamp --stamp-docs "$REPO"
expect_no_reason "documentation (" "docs stamp matches under configured paths" "git commit -m x"
configure '*.rst'
stage code.py; stage guide.rst
expect_no_reason "documentation (" "non-markdown docs (*.rst) satisfy the docs check" "git commit -m x"
git -C "$REPO" reset -q guide.rst; stage README.md
expect_reason "documentation (" "markdown is code when only *.rst counts as docs" "git commit -m x"
configure '# nothing configured\n'
stage code.py; stage README.md
expect_no_reason "documentation (" "comment-only file falls back to *.md" "git commit -m x"
configure 'site/'
stage code.py
expect_no_reason "documentation (" "no tracked file under docs paths: docs check off" "git commit -m x"
configure ':(glob)docs/**'
stage code.py
stamp --stamp-review "$REPO"
stage docs/a/deep.md
expect allow "long-form pathspec magic is excluded from the code-only hash" "git commit -m x"
expect_no_reason "documentation (" "long-form pathspec magic counts as docs" "git commit -m x"

[ "$fails" -eq 0 ] && echo "commit-gate: all passed" || echo "commit-gate: $fails failed"
[ "$fails" -eq 0 ]
