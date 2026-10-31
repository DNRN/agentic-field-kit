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

echo "# non-commit calls skip the parser"
# A python3 shim on PATH records each launch, then runs the real interpreter.
SHIM="$WORK/shim"; mkdir -p "$SHIM"
printf '#!/bin/sh\necho launched >> "%s/launches"\nexec %s "$@"\n' "$SHIM" "$(command -v python3)" > "$SHIM/python3"
chmod +x "$SHIM/python3"
launches() { # $1=command -> number of python3 launches during one gate run
  rm -f "$SHIM/launches"
  python3 -c 'import json,sys; print(json.dumps({"tool_input":{"command":sys.argv[1]},"cwd":sys.argv[2]}))' "$1" "$REPO" |
    env -i PATH="$SHIM:/usr/bin:/bin" bash "$GATE" >/dev/null
  if [ -f "$SHIM/launches" ]; then wc -l < "$SHIM/launches" | tr -d ' '; else echo 0; fi
}
n="$(launches "ls -la && git status")"
if [ "$n" -eq 0 ]; then echo "ok   non-commit command never launches python3"; else echo "FAIL non-commit command launched python3 $n time(s)"; fails=$((fails + 1)); fi
n="$(launches "git commit -m x")"
if [ "$n" -gt 0 ]; then echo "ok   commit command does launch python3 (shim is effective)"; else echo "FAIL shim saw no launch for a commit"; fails=$((fails + 1)); fi

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

echo "# message names skills and stamp as installed"
new_repo; stage code.py
expect_reason "/afk:code-review" "plugin name without a project install" "git commit -m x"
mkdir -p "$REPO/.agents/skills/code-review" "$REPO/.agents/skills/doc-sync" "$REPO/.agents/hooks"
cp "$GATE" "$REPO/.agents/hooks/commit-gate.sh"
expect_reason "run the code-review skill" "bare skill name with a project install" "git commit -m x"
expect_reason "run the doc-sync skill" "bare doc-sync name with a project install" "git commit -m x"
expect_reason "$REPO/.agents/hooks/commit-gate.sh --stamp-review" "stamp command uses the installed copy" "git commit -m x"
expect_no_reason "/afk:" "no plugin names with a project install" "git commit -m x"

echo "# --pre-commit (git hook mode)"
# hook_repo: a fresh repo whose pre-commit hook runs the gate, as the shim does.
hook_repo() {
  new_repo
  printf '#!/bin/sh\nexec bash "%s" --pre-commit\n' "$GATE" > "$REPO/.git/hooks/pre-commit"
  chmod +x "$REPO/.git/hooks/pre-commit"
}
commit() { # [git commit args...] -> commit's stderr; exit status is commit's
  (cd "$REPO" && env -i PATH=/usr/bin:/bin HOME="$WORK" git commit -q "$@" 2>&1 >/dev/null)
}
expect_commit() { # $1=ok|denied $2=description, rest: git commit args
  local want="$1" desc="$2" out got; shift 2
  if out="$(commit "$@")"; then got=ok; else got=denied; fi
  if [ "$got" = "$want" ]; then echo "ok   $desc"; else echo "FAIL $desc: expected $want, got $got: $out"; fails=$((fails + 1)); fi
  LAST="$out"
}
expect_last() { # $1=substring $2=description
  case "$LAST" in *"$1"*) echo "ok   $2" ;; *) echo "FAIL $2: stderr lacks '$1': $LAST"; fails=$((fails + 1)) ;; esac
}
hook_repo; stage code.py; stage README.md
expect_commit denied "unreviewed staged change is denied" -m x
expect_last "code review (unreviewed changes)" "stderr names the pending review"
expect_last "Ask the user" "stderr tells the agent to ask"
expect_last "--stamp-review $REPO" "stderr names the stamp command"
change code.py
expect_commit denied "partial staging still denied" -m x
expect_last "--stamp-review $REPO" "partial staging names the staged stamp"
git -C "$REPO" checkout -q code.py
stamp --stamp-review "$REPO"
expect_commit ok "stamped diff is committed" -m x
stage code.py; stage README.md
expect_commit ok "--no-verify bypasses" --no-verify -m x
stage code.py; stage README.md
expect_commit ok "-n bypasses" -n -m x

hook_repo; change code.py; change README.md
expect_commit denied "commit -a unreviewed is denied" -a -m x
expect_last "--stamp-review --all $REPO" "commit -a names the --all stamp"
stamp --stamp-review --all "$REPO"
expect_commit ok "--all stamp matches the hook's staged diff under commit -a" -a -m x

hook_repo; stage code.py
expect_commit denied "code without docs is denied" -m x
expect_last "documentation (" "stderr names the docs check"
stamp --stamp-review "$REPO"; stamp --stamp-docs "$REPO"
expect_commit ok "review and docs stamps clear both checks" -m x
stage code.py; stage README.md
stamp --stamp-review "$REPO"
expect_commit ok "doc edit in the commit satisfies the docs check" -m x

hook_repo
expect_commit ok "empty commit passes" --allow-empty -m x
stage code.py; stage README.md
printf '\377\376garbage\n' > "$REPO/.commit-gate-docs"
stamp --stamp-review "$REPO"
expect_commit ok "unusual .commit-gate-docs does not wedge a stamped commit" -m x

hook_repo; stage code.py
mkdir -p "$REPO/.agents/skills/code-review"
expect_commit denied "project install message" -m x
expect_last "run the code-review skill" "hook message names the installed skill"
out="$(cd "$WORK" && env -i PATH=/usr/bin:/bin bash "$GATE" --pre-commit 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && [ -z "$out" ]; then echo "ok   outside a repository: silent allow"; else echo "FAIL outside a repository: rc=$rc out=$out"; fails=$((fails + 1)); fi
out="$(cd "$REPO" && git reset -q && env -i PATH=/usr/bin:/bin bash "$GATE" --pre-commit 2>&1)"; rc=$?
if [ "$rc" -eq 0 ] && [ -z "$out" ]; then echo "ok   nothing staged: silent allow"; else echo "FAIL nothing staged: rc=$rc out=$out"; fails=$((fails + 1)); fi

echo "# adr-guard (--pre-commit)"
# adr <file> <status> [crlf]: an ADR with a Status section and a body.
adr() {
  local nl='\n'; [ "${3:-}" = crlf ] && nl='\r\n'
  mkdir -p "$REPO/docs/adr"
  printf "# 0001. Use X${nl}${nl}Date: 2026-10-30${nl}${nl}## Status${nl}${nl}%s${nl}${nl}## Context${nl}${nl}Forces.${nl}${nl}## Decision${nl}${nl}We will use X.${nl}" "$2" > "$REPO/docs/adr/$1"
}
adr_repo() { # fresh hook repo with committed ADRs; review stamps are made per test
  hook_repo
  adr 0001-accepted.md Accepted
  adr 0002-proposed.md Proposed
  adr 0003-crlf.md Accepted crlf
  adr 0004-partial.md "Accepted. Item 2 superseded by [0009](0009-x.md)."
  printf '# Not an ADR\n\n## Status\n\nAccepted\n' > "$REPO/docs/adr/README.md"
  printf '# 0005. Odd\n\n**Status:** Accepted\n\nBody.\n' > "$REPO/docs/adr/0005-odd.md"
  git -C "$REPO" add -A; git -C "$REPO" commit -qm adrs --no-verify
}
# adr_commit <ok|denied> <description>: stamp review and docs for what is
# staged, so only the ADR check can deny, then commit.
adr_commit() {
  stamp --stamp-review "$REPO"; stamp --stamp-docs "$REPO"
  expect_commit "$1" "$2" -m x
}
adr_repo
sed -i.bak 's/Forces\./Other forces./' "$REPO/docs/adr/0001-accepted.md"; git -C "$REPO" add docs/adr/0001-accepted.md
adr_commit denied "editing an Accepted ADR's Context is denied"
expect_last "docs/adr/0001-accepted.md (edited outside Status)" "the denial names the ADR"
expect_last "superseding ADR" "the denial says to supersede"
expect_last "--no-verify" "the denial names the waiver"
expect_commit ok "--no-verify waives it" --no-verify -m x

adr_repo
sed -i.bak 's/^Accepted$/Superseded by [0016](0016-y.md)/' "$REPO/docs/adr/0001-accepted.md"; git -C "$REPO" add docs/adr/0001-accepted.md
adr_commit ok "changing only the Status to Superseded by is allowed"
adr_repo
adr 0001-accepted.md "$(printf 'Accepted\n\nPlan: [p](../plans/p.md). Item 1 superseded by 0016.')"; git -C "$REPO" add docs/adr/0001-accepted.md
adr_commit ok "adding a note under Status is allowed"
sed -i.bak 's/We will use X\./We will use Y./' "$REPO/docs/adr/0004-partial.md"; git -C "$REPO" add docs/adr/0004-partial.md
adr_commit denied "a partly superseded ADR (Accepted. ...) is still guarded"

adr_repo
sed -i.bak 's/Forces\./Other forces./' "$REPO/docs/adr/0002-proposed.md"; git -C "$REPO" add docs/adr/0002-proposed.md
adr_commit ok "editing a Proposed ADR is allowed"
sed -i.bak 's/^Proposed$/Accepted/' "$REPO/docs/adr/0002-proposed.md"; git -C "$REPO" add docs/adr/0002-proposed.md
adr_commit ok "accepting a Proposed ADR is allowed"

adr_repo
git -C "$REPO" rm -q docs/adr/0001-accepted.md
adr_commit denied "deleting an Accepted ADR is denied"
expect_last "0001-accepted.md (deleted)" "the denial says deleted"
adr_repo
git -C "$REPO" mv docs/adr/0001-accepted.md docs/adr/0001-renamed.md
adr_commit denied "renaming an Accepted ADR is denied"
expect_last "0001-accepted.md (renamed to docs/adr/0001-renamed.md)" "the denial names both paths"

adr_repo
adr 0006-new.md Accepted; git -C "$REPO" add docs/adr/0006-new.md
adr_commit ok "a new ADR is allowed, even Accepted"
sed -i.bak 's/Accepted/Rejected/' "$REPO/docs/adr/README.md"; git -C "$REPO" add docs/adr/README.md
adr_commit ok "files that aren't NNNN-*.md are not ADRs"
sed -i.bak 's/Body\./Changed./' "$REPO/docs/adr/0005-odd.md"; git -C "$REPO" add docs/adr/0005-odd.md
adr_commit ok "an unknown Status format fails open"

adr_repo
adr 0003-crlf.md "Superseded by 0016" crlf; git -C "$REPO" add docs/adr/0003-crlf.md
adr_commit ok "CRLF ADR: Status-only change allowed"
sed -i.bak 's/Forces\./Other./' "$REPO/docs/adr/0003-crlf.md"; git -C "$REPO" add docs/adr/0003-crlf.md
adr_commit ok "CRLF ADR, no longer Accepted at HEAD: edit allowed"
adr_repo
tr -d '\r' < "$REPO/docs/adr/0003-crlf.md" > "$REPO/x" && mv "$REPO/x" "$REPO/docs/adr/0003-crlf.md"; git -C "$REPO" add docs/adr/0003-crlf.md
adr_commit ok "CRLF to LF conversion is not an edit"
sed -i.bak 's/Forces\./Other./' "$REPO/docs/adr/0003-crlf.md"; git -C "$REPO" add docs/adr/0003-crlf.md
adr_commit denied "CRLF Accepted ADR: body edit denied"

adr_repo
stage code.py
sed -i.bak 's/Forces\./Other forces./' "$REPO/docs/adr/0001-accepted.md"; git -C "$REPO" add docs/adr/0001-accepted.md
expect_commit denied "unreviewed change plus ADR edit" -m x
expect_last "code review (unreviewed changes)" "one denial lists the review check"
expect_last "Accepted ADRs are immutable" "and the ADR check"

hook_repo
mkdir -p "$REPO/docs/adr"; adr 0001-a.md Accepted; git -C "$REPO" add -A
out="$(cd "$WORK" && rm -rf first && git init -q first && cd first && cp -R "$REPO/docs" . && git add -A && env -i PATH=/usr/bin:/bin bash "$GATE" --pre-commit 2>&1)"
case "$out" in *"immutable"*) echo "FAIL first commit (no HEAD) must not trip the ADR check"; fails=$((fails + 1)) ;; *) echo "ok   first commit (no HEAD): no ADR check" ;; esac

echo "# one gate at a time"
# install_copy: the project's copy of the gate, as the installer writes it.
install_copy() { mkdir -p "$REPO/.agents/hooks"; cp "$GATE" "$REPO/.agents/hooks/commit-gate.sh"; }
# write_hook <path> <content>: an executable pre-commit hook.
write_hook() { mkdir -p "$(dirname "$1")"; printf '%b' "$2" > "$1"; chmod +x "$1"; }
PRE_COMMIT_SHIM='#!/bin/sh\n# afk commit-gate\ngate="$(git rev-parse --show-toplevel)/.agents/hooks/commit-gate.sh"\n[ -f "$gate" ] || exit 0\nexec bash "$gate" --pre-commit\n'
new_repo; stage code.py; install_copy
expect deny "copy without a git hook: plugin still asks" "git commit -m x"
write_hook "$REPO/.git/hooks/pre-commit" "$PRE_COMMIT_SHIM"
expect allow "shim installed: plugin defers to the git hook" "git commit -m x"
expect allow "shim installed: commit -a also deferred" "git commit -am x"
expect allow "shim installed: --no-verify still allowed" "git commit --no-verify -m x"
expect allow "shim installed: deferred from a subdirectory" "git commit -m x" "$REPO/sub"
expect_commit denied "plugin and git hook together: the git hook asks" -m x
expect_last "commit-gate: Pending" "the one ask comes from the git hook"
chmod -x "$REPO/.git/hooks/pre-commit"
expect deny "shim not executable (git skips it): plugin asks" "git commit -m x"
chmod +x "$REPO/.git/hooks/pre-commit"
rm "$REPO/.agents/hooks/commit-gate.sh"
expect deny "shim without the copy (shim fails open): plugin asks" "git commit -m x"
install_copy
write_hook "$REPO/.git/hooks/pre-commit" '#!/bin/sh\nnpx lint-staged\n'
expect deny "foreign pre-commit hook: plugin asks" "git commit -m x"
write_hook "$REPO/.git/hooks/pre-commit" '#!/bin/sh\nnpx lint-staged\nbash .agents/hooks/commit-gate.sh --pre-commit || exit 1\n'
expect allow "foreign hook with the one-line addition: plugin defers" "git commit -m x"
rm "$REPO/.git/hooks/pre-commit"
git -C "$REPO" config core.hooksPath .husky
write_hook "$REPO/.husky/pre-commit" 'bash .agents/hooks/commit-gate.sh --pre-commit || exit 1\n'
expect allow "core.hooksPath (relative) is honoured" "git commit -m x"
git -C "$REPO" config core.hooksPath "$WORK/abs-hooks"
expect deny "core.hooksPath elsewhere: .husky hook no longer runs, plugin asks" "git commit -m x"
write_hook "$WORK/abs-hooks/pre-commit" "$PRE_COMMIT_SHIM"
expect allow "core.hooksPath (absolute) is honoured" "git commit -m x"

[ "$fails" -eq 0 ] && echo "commit-gate: all passed" || echo "commit-gate: $fails failed"
[ "$fails" -eq 0 ]
