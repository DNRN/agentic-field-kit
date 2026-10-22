#!/usr/bin/env bash
# Regression tests for install.sh: it must never replace or remove anything it
# did not install.
set -u
INSTALL="$(cd "$(dirname "$0")/.." && pwd)/install.sh"
HOME="$(mktemp -d)"
export HOME
trap 'rm -rf "$HOME"' EXIT
SKILLS="$HOME/.claude/skills"
fails=0
check() { # $1=description, rest=test command
  local d="$1"; shift
  if "$@"; then echo "ok   $d"; else echo "FAIL $d"; fails=$((fails + 1)); fi
}
install() { "$INSTALL" --harness claude "$@" >/dev/null; }

mkdir -p "$SKILLS/code-review" "$HOME/elsewhere/doc-sync"
echo mine > "$SKILLS/code-review/notes.txt"
ln -s "$HOME/elsewhere/doc-sync" "$SKILLS/doc-sync"
ln -s "$HOME/gone" "$SKILLS/adr-planning"   # dangling user symlink

for mode in copy symlink; do
  install --mode "$mode"; rc=$?
  check "$mode: reports skips with non-zero exit" [ "$rc" -ne 0 ]
  check "$mode: user's real directory untouched" [ -f "$SKILLS/code-review/notes.txt" ]
  check "$mode: user's symlink untouched" [ "$(readlink "$SKILLS/doc-sync")" = "$HOME/elsewhere/doc-sync" ]
  check "$mode: user's dangling symlink untouched" [ "$(readlink "$SKILLS/adr-planning")" = "$HOME/gone" ]
done

"$INSTALL" --harness claude --remove >/dev/null
check "remove: user's real directory survives" [ -f "$SKILLS/code-review/notes.txt" ]
check "remove: user's symlink survives" [ -L "$SKILLS/doc-sync" ]

rm -rf "$SKILLS"
install --mode copy
check "copy: installs with marker" [ -f "$SKILLS/code-review/.agentic-toolkit-install" ]
install --mode symlink
check "symlink: replaces its own copy" [ -L "$SKILLS/code-review" ]
install --mode copy
check "copy: replaces its own symlink" [ ! -L "$SKILLS/code-review" ]
"$INSTALL" --harness claude --remove >/dev/null
check "remove: own installs removed" [ ! -e "$SKILLS/code-review" ]

[ "$fails" -eq 0 ] && echo "install: all passed" || echo "install: $fails failed"
[ "$fails" -eq 0 ]
