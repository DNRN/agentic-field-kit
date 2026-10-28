#!/usr/bin/env bash
# Regression tests for install.sh (plan: docs/plans/selective-install.md).
# It must never replace or remove anything it did not install, must not delete
# local edits without --force, and must keep project installs consistent with
# their manifest so that `git pull` + `install.sh --update` is the whole update.
# shellcheck disable=SC2016 # holds evaluates its single-quoted conditions later
set -u
SRC="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"
mkdir -p "$HOME"

# A private toolkit clone, so tests can change components "upstream".
C="$TMP/toolkit"
mkdir -p "$C"
cp -R "$SRC/install.sh" "$SRC/catalog.tsv" "$SRC/skills" "$SRC/agents" "$SRC/hooks" "$SRC/.claude-plugin" "$C/"
I="$C/install.sh"

TAB=$'\t'
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) WINDOWS=1 ;; *) WINDOWS=0 ;; esac
default_form() { # path → it has the global default form: a copy on Windows, else a link
  if [ "$WINDOWS" = 1 ]; then [ -d "$1" ] && [ ! -L "$1" ]; else [ -L "$1" ]; fi
}
fails=0
check() { # $1=description, rest=test command
  local d="$1"; shift
  if "$@"; then echo "ok   $d"; else echo "FAIL $d"; fails=$((fails + 1)); fi
}
holds() { eval "$1"; } # a condition written as one string, so && stays inside check
exists_link() { [ -L "$1" ] || [ -e "$1" ]; }
project() { # fresh git repository → path
  local p="$TMP/$1"
  rm -rf "$p"; mkdir -p "$p"; git -C "$p" init -q; printf '%s' "$p"
}
st() { # scope-column(project|global) name [install.sh args] → status from --list
  local col=$1 n=$2; shift 2
  "$I" --list "$@" | awk -v n="$n" -v c="$([ "$col" = project ] && echo 4 || echo 5)" '$2 == n { print $c }'
}
row() { cut -f2,5 "$1/.agents/agentic-toolkit.tsv" 2>/dev/null | grep -x "$2" >/dev/null; }
in_manifest() { cut -f2 "$1/.agents/agentic-toolkit.tsv" 2>/dev/null | grep -qx "$2"; }

# ---- global scope: the rules install.sh always had ---------------------------
G="$HOME/.claude/skills"
mkdir -p "$G/code-review" "$HOME/elsewhere/doc-sync"
echo mine > "$G/code-review/notes.txt"
ln -s "$HOME/elsewhere/doc-sync" "$G/doc-sync"
mkdir -p "$HOME/gone" && ln -s "$HOME/gone" "$G/adr-planning" && rmdir "$HOME/gone"   # dangling user symlink
for mode in copy symlink; do
  "$I" --global --harness claude --mode "$mode" --all >/dev/null; rc=$?
  check "global $mode: reports skips with non-zero exit" [ "$rc" -ne 0 ]
  check "global $mode: user's real directory untouched" [ -f "$G/code-review/notes.txt" ]
  check "global $mode: user's symlink untouched" [ "$(readlink "$G/doc-sync")" = "$HOME/elsewhere/doc-sync" ]
  check "global $mode: user's dangling symlink untouched" [ "$(readlink "$G/adr-planning")" = "$HOME/gone" ]
done
check "global: someone else's skill is listed as foreign" [ "$(st global code-review --global)" = foreign ]
"$I" --global --harness claude --remove >/dev/null
check "global remove: user's real directory survives" [ -f "$G/code-review/notes.txt" ]
check "global remove: user's symlink survives" [ -L "$G/doc-sync" ]

rm -rf "$G"
"$I" --global --harness claude --mode copy --all >/dev/null
check "global copy: recorded in ~/.agents/agentic-toolkit.tsv" in_manifest "$HOME" code-review
check "global copy: listed as installed" [ "$(st global code-review --global)" = installed ]
"$I" --global --harness claude --mode symlink --all >/dev/null
check "global symlink: replaces its own copy" [ -L "$G/code-review" ]
check "global symlink: listed as linked" [ "$(st global code-review --global)" = linked ]
"$I" --global --harness claude --mode copy --all >/dev/null
check "global copy: replaces its own symlink" [ ! -L "$G/code-review" ]
"$I" --global --harness claude --remove >/dev/null
check "global remove: own installs removed" [ ! -e "$G/code-review" ]
check "global remove: manifest emptied" [ ! -e "$HOME/.agents/agentic-toolkit.tsv" ]

"$I" --global --all >/dev/null
check "global default: into ~/.agents/skills, linked (copied on Windows)" default_form "$HOME/.agents/skills/code-review"
check "global default: leaves ~/.claude/skills alone (would hide the plugin)" [ ! -e "$G/code-review" ]
check "global: agents are not installed globally" [ ! -e "$HOME/.claude/agents" ]
"$I" --global --remove >/dev/null
check "global remove: clears ~/.agents/skills" [ ! -e "$HOME/.agents/skills/code-review" ]

# Installs made before the manifest existed are still ours.
mkdir -p "$G" "$HOME/.omp/agent/skills"
ln -s "$C/skills/code-review/" "$G/code-review"
cp -R "$C/skills/doc-sync" "$HOME/.omp/agent/skills/doc-sync"
echo "$C" > "$HOME/.omp/agent/skills/doc-sync/.agentic-toolkit-install"
check "legacy link: listed as linked" [ "$(st global code-review --global)" = linked ]
check "legacy copy: listed as installed" [ "$(st global doc-sync --global)" = installed ]
"$I" --global --remove >/dev/null
check "legacy: --global --remove clears old links" [ ! -L "$G/code-review" ]
check "legacy: --global --remove clears old copies" [ ! -e "$HOME/.omp/agent/skills/doc-sync" ]
"$I" --global adr-planning >/dev/null
mv "$C/skills/adr-planning" "$TMP/adr-planning.moved"
check "global: our symlink to a vanished folder is dangling" [ "$(st global adr-planning --global)" = dangling ]
mv "$TMP/adr-planning.moved" "$C/skills/adr-planning"
"$I" --global --remove >/dev/null
mkdir -p "$G"
mkdir -p "$TMP/other-clone/skills/adr-planning"
ln -s "$TMP/other-clone/skills/adr-planning" "$G/adr-planning"
rm -rf "$TMP/other-clone"
"$I" --global --remove >/dev/null
check "global remove: a link into another folder survives" [ -L "$G/adr-planning" ]
rm -rf "$G"

# ---- project scope -----------------------------------------------------------
P=$(project shop)
mkdir -p "$P/src"
(cd "$P/src" && "$I" code-reviewer >/dev/null)
check "project: default scope is the git root, also from a subdirectory" [ -f "$P/.agents/skills/code-review/SKILL.md" ]
check "project: skills are copies, not links" [ ! -L "$P/.agents/skills/code-review" ]
check "project: Claude Code gets its own copy" holds '[ -f "$P/.claude/skills/code-review/SKILL.md" ] && [ ! -L "$P/.claude/skills/code-review" ]'
check "project: agent copied into .claude/agents" [ -f "$P/.claude/agents/code-reviewer.md" ]
check "project: a requirement is installed and recorded as one" row "$P" "code-review${TAB}dependency"
check "project: the named component is recorded as explicit" row "$P" "code-reviewer${TAB}explicit"
check "project: nothing written outside the project" [ ! -e "$HOME/.agents/skills/code-review" ]
check "project: listed as installed" [ "$(cd "$P" && st project code-review)" = installed ]
check "project: not installed is -" [ "$(cd "$P" && st project doc-sync)" = - ]
check "project: hooks are plugin only (n/a)" [ "$(cd "$P" && st project commit-gate)" = n/a ]
(cd "$P" && "$I" code-review >/dev/null)
check "project: naming an installed requirement makes it explicit" row "$P" "code-review${TAB}explicit"
(cd "$P" && "$I" --remove code-reviewer >/dev/null)
check "remove NAME: removes only that component" holds '[ ! -e "$P/.claude/agents/code-reviewer.md" ] && [ -d "$P/.agents/skills/code-review" ]'

P=$(project deps)
cd "$P" || exit 1
"$I" code-reviewer >/dev/null
"$I" --remove code-review >/dev/null 2>&1; rc=$?
check "remove: refuses a requirement of an installed component" holds '[ "$rc" -ne 0 ] && [ -d "$P/.agents/skills/code-review" ]'
"$I" --remove code-reviewer >/dev/null
check "remove without --yes: keeps the requirement it would orphan" in_manifest "$P" code-review
"$I" code-reviewer >/dev/null
"$I" --remove code-reviewer --yes >/dev/null
check "remove --yes: also removes the orphaned requirement" [ ! -e "$P/.agents/skills/code-review" ]
check "remove: manifest and empty folders cleaned up" holds '[ ! -e "$P/.agents" ] && [ ! -e "$P/.claude" ]'
cd "$TMP" || exit 1

"$I" adr-planning >/dev/null 2>&1; rc=$?
check "project: refuses outside a git repository" [ "$rc" -ne 0 ]
(cd "$C" && git init -q && "$I" adr-planning >/dev/null 2>&1); rc=$?
check "project: refuses the toolkit clone itself" holds '[ "$rc" -ne 0 ] && [ ! -e "$C/.agents" ]'
rm -rf "$C/.git"
mkdir -p "$TMP/plain"
"$I" --project "$TMP/plain" adr-planning >/dev/null
check "project: --project DIR works outside git" in_manifest "$TMP/plain" adr-planning
P=$(project quiet)
(cd "$P" && "$I" </dev/null >/dev/null 2>&1); rc=$?
check "no names and no terminal: error, nothing installed" holds '[ "$rc" -ne 0 ] && [ ! -e "$P/.agents" ]'
(cd "$P" && "$I" --mode symlink adr-planning >/dev/null 2>&1); rc=$?
check "project: --mode is rejected (project installs are copies)" [ "$rc" -ne 0 ]

P=$(project foreign)
mkdir -p "$P/.claude/agents"
echo mine > "$P/.claude/agents/doc-auditor.md"
(cd "$P" && "$I" doc-auditor >/dev/null); rc=$?
check "project: someone else's file is skipped, with non-zero exit" holds '[ "$rc" -ne 0 ] && [ "$(cat "$P/.claude/agents/doc-auditor.md")" = mine ]'
check "project: someone else's file is listed as foreign" [ "$(cd "$P" && st project doc-auditor)" = foreign ]
(cd "$P" && "$I" --remove --yes >/dev/null)
check "project: remove-all leaves someone else's file alone" [ -f "$P/.claude/agents/doc-auditor.md" ]

P=$(project plugin)
mkdir -p "$P/.claude"
printf '{"enabledPlugins": {"agentic-toolkit@agentic-toolkit": true}}\n' > "$P/.claude/settings.json"
(cd "$P" && "$I" --all >/dev/null)
check "plugin enabled for the project: skills only in .agents" holds '[ -d "$P/.agents/skills/code-review" ] && [ ! -e "$P/.claude/skills" ]'
check "plugin enabled for the project: agents not copied" [ ! -e "$P/.claude/agents" ]
check "plugin enabled for the project: agents listed as plugin" [ "$(cd "$P" && st project code-reviewer)" = plugin ]

# ---- updates: git pull in the clone, then --update ---------------------------
P=$(project upd)
cd "$P" || exit 1
"$I" code-review doc-sync adr-planning >/dev/null
echo "upstream change" >> "$C/skills/code-review/SKILL.md"
echo "local edit" >> "$P/.claude/skills/doc-sync/SKILL.md"
check "update: toolkit change shows as outdated" [ "$(st project code-review)" = outdated ]
check "update: local edit shows as modified" [ "$(st project doc-sync)" = modified ]
"$I" --update >/dev/null
check "update: outdated copies refreshed" grep -q "upstream change" "$P/.agents/skills/code-review/SKILL.md"
check "update: second copy refreshed too" grep -q "upstream change" "$P/.claude/skills/code-review/SKILL.md"
check "update: listed as installed afterwards" [ "$(st project code-review)" = installed ]
check "update: local edit kept without --force" grep -q "local edit" "$P/.claude/skills/doc-sync/SKILL.md"
"$I" --remove doc-sync >/dev/null
check "remove: local edit kept without --force" [ -d "$P/.claude/skills/doc-sync" ]
"$I" --update --force >/dev/null
check "update --force: overwrites the local edit" [ "$(st project doc-sync)" = installed ]
rm -rf "$P/.claude/skills/adr-planning"
check "update: deleted copy shows as missing" [ "$(st project adr-planning)" = missing ]
"$I" --update >/dev/null
check "update: restores the missing copy" [ -f "$P/.claude/skills/adr-planning/SKILL.md" ]
cp "$C/catalog.tsv" "$TMP/catalog.orig"
"$I" --remove adr-planning >/dev/null
sed -i.bak 's/^\(skill	code-review	skills\/code-review	all	\)-/\1adr-planning/' "$C/catalog.tsv"
"$I" --update >/dev/null
check "update: installs a requirement added upstream" row "$P" "adr-planning${TAB}dependency"
cp "$TMP/catalog.orig" "$C/catalog.tsv"
sed -i.bak '/^skill	adr-planning/d' "$C/catalog.tsv"
check "update: component dropped upstream shows as retired" [ "$(st project adr-planning)" = retired ]
"$I" --update >/dev/null
check "update without --yes: keeps a retired component" [ -d "$P/.agents/skills/adr-planning" ]
"$I" --update --yes >/dev/null
check "update --yes: removes a retired component" holds '[ ! -e "$P/.agents/skills/adr-planning" ] && ! in_manifest "$P" adr-planning'
cp "$TMP/catalog.orig" "$C/catalog.tsv"
rm -f "$C/catalog.tsv.bak"
cd "$TMP" || exit 1

# ---- edits, hostile manifests, odd roots -------------------------------------
P=$(project edits)
cd "$P" || exit 1
"$I" code-review >/dev/null
echo "local edit" >> "$P/.agents/skills/code-review/SKILL.md"
rm -rf "$P/.claude/skills/code-review"
check "an edit next to a deleted copy still shows as modified" [ "$(st project code-review)" = modified ]
"$I" --update >/dev/null
check "update: keeps an edit even when the other copy was deleted" grep -q "local edit" "$P/.agents/skills/code-review/SKILL.md"
"$I" --remove code-review >/dev/null
check "remove: keeps an edit even when the other copy was deleted" grep -q "local edit" "$P/.agents/skills/code-review/SKILL.md"

P=$(project crlf-manifest)
cd "$P" || exit 1
"$I" code-review >/dev/null
sed -i.bak 's/$/\r/' "$P/.agents/agentic-toolkit.tsv" && rm -f "$P/.agents/agentic-toolkit.tsv.bak"
check "a CRLF manifest still reads as installed" [ "$(st project code-review)" = installed ]
"$I" --update >/dev/null
check "update with a CRLF manifest creates no stray folders" [ "$(ls "$P/.claude/skills")" = code-review ]

P=$(project hostile)
cd "$P" || exit 1
mkdir -p "$TMP/victim" && echo keep > "$TMP/victim/data.txt"
"$I" code-review >/dev/null
sed -i.bak "s|\.claude/skills/code-review\$|.claude/skills/code-review ../victim|" "$P/.agents/agentic-toolkit.tsv"
rm -rf "$P/.claude/skills/code-review"
"$I" --update >/dev/null
"$I" --remove --force --yes >/dev/null
check "a manifest path outside the component's folders is never touched" [ "$(cat "$TMP/victim/data.txt")" = keep ]
"$I" code-review >/dev/null
sed -i.bak "s|\.agents/skills/code-review \.claude/skills/code-review\$|../x|" "$P/.agents/agentic-toolkit.tsv"
check "a manifest row that claims no valid path claims nothing" [ "$(st project code-review)" = foreign ]

P=$(project unavailable)
cd "$P" || exit 1
"$I" commit-gate >/dev/null 2>&1; rc=$?
check "naming a component that can't be installed here fails and installs nothing" holds '[ "$rc" -ne 0 ] && [ ! -e "$P/.agents" ]'

"$I" code-reviewer adr-planning >/dev/null
echo "local edit" >> "$P/.claude/agents/code-reviewer.md"
"$I" --remove >/dev/null 2>&1
check "remove: an edited component keeps what it requires" holds '[ -d "$P/.agents/skills/code-review" ] && [ -f "$P/.claude/agents/code-reviewer.md" ]'
check "remove: and still removes the rest" [ ! -e "$P/.agents/skills/adr-planning" ]
cd "$TMP" || exit 1

git -C "$HOME" init -q && mkdir -p "$HOME/notes"
(cd "$HOME/notes" && "$I" adr-planning >/dev/null 2>&1); rc=$?
check "project: refuses a git root that is the home directory (the global scope)" holds '[ "$rc" -ne 0 ] && ! in_manifest "$HOME" adr-planning'
rm -rf "$HOME/.git" "$HOME/notes"

"$I" --global --mode copy code-review >/dev/null
echo "another upstream change" >> "$C/skills/code-review/SKILL.md"
"$I" --global --update >/dev/null
check "global update: a copy stays a copy" holds '[ -d "$HOME/.agents/skills/code-review" ] && [ ! -L "$HOME/.agents/skills/code-review" ]'
rm -rf "$HOME/.agents/skills/code-review"
"$I" --global --update >/dev/null
check "global update: a deleted copy comes back as a copy" holds '[ -d "$HOME/.agents/skills/code-review" ] && [ ! -L "$HOME/.agents/skills/code-review" ]'
"$I" --global --remove >/dev/null
"$I" --global --mode symlink code-review >/dev/null
rm -f "$HOME/.agents/skills/code-review"
"$I" --global --update >/dev/null
check "global update: a deleted link comes back as a link" [ -L "$HOME/.agents/skills/code-review" ]
"$I" --global --remove >/dev/null
"$I" --global adr-planning >/dev/null
cp "$C/catalog.tsv" "$TMP/catalog.orig"
sed -i.bak '/^skill	adr-planning/d' "$C/catalog.tsv"
mv "$C/skills/adr-planning" "$TMP/adr-planning.moved"
"$I" --global --update --yes >/dev/null; rc=$?
check "global update --yes: removes a retired skill whose link now dangles" holds '[ "$rc" -eq 0 ] && ! exists_link "$HOME/.agents/skills/adr-planning"'
mv "$TMP/adr-planning.moved" "$C/skills/adr-planning"
cp "$TMP/catalog.orig" "$C/catalog.tsv"
rm -f "$C/catalog.tsv.bak"

# ---- line endings, teammates, both scopes ------------------------------------
P=$(project crlf)
(cd "$P" && "$I" code-review >/dev/null)
sed -i.bak 's/$/\r/' "$P/.agents/skills/code-review/SKILL.md" && rm -f "$P/.agents/skills/code-review/SKILL.md.bak"
check "a CRLF checkout of a copy is not modified" [ "$(cd "$P" && st project code-review)" = installed ]

git -C "$P" add -A && git -C "$P" -c user.name=t -c user.email=t@t commit -qm skills
git clone -q "$P" "$TMP/teammate"
check "teammate clone: committed copies are installed" [ "$(cd "$TMP/teammate" && st project code-review)" = installed ]

"$I" --global code-review >/dev/null
check "both scopes: --list warns" bash -c "cd '$P' && '$I' --list | grep -q '! in both scopes'"
out=$(cd "$P" && "$I" code-review doc-sync </dev/null 2>&1)
check "both scopes: install warns" grep -q "also installed globally" <<<"$out"
"$I" --global --remove >/dev/null

# ---- interactive menu ----------------------------------------------------------
P=$(project menu)
cd "$P" || exit 1
printf '3\n\ny\n' | "$I" --interactive >/dev/null
check "menu: mark, review, apply installs" [ -d "$P/.agents/skills/doc-sync" ]
printf '3\n\ny\n' | "$I" --interactive >/dev/null
check "menu: marking an installed component removes it" [ ! -e "$P/.agents/skills/doc-sync" ]
printf '1 2\nq\n' | "$I" --interactive >/dev/null
check "menu: q quits without changes" [ ! -e "$P/.agents" ]
printf '1\n\nn\nq\n' | "$I" --interactive >/dev/null
check "menu: declining the review changes nothing" [ ! -e "$P/.agents" ]
printf 'g\n1\n\ny\n' | "$I" --interactive >/dev/null
check "menu: g switches to the global scope" holds 'default_form "$HOME/.agents/skills/adr-planning" && [ ! -e "$P/.agents" ]'
out=$(printf '6\nq\n' | "$I" --interactive)
check "menu: says why the hook can't be selected" grep -q "commit-gate: plugin only until roadmap phase 1 (portable" <<<"$out"
out=$(printf '0\nq\n' | "$I" --interactive); rc=$?
check "menu: 0 is rejected, not fatal" holds '[ "$rc" -eq 0 ] && grep -q "Not a component number: 0" <<<"$out"'
"$I" code-reviewer >/dev/null
out=$(printf '4\n\ny\n' | "$I" --interactive)
check "menu: the review lists the requirement that applying removes" grep -q "remove  .agents/skills/code-review" <<<"$out"
check "menu: applying removes it" [ ! -e "$P/.agents/skills/code-review" ]
cd "$TMP" || exit 1

[ "$fails" -eq 0 ] && echo "install: all passed" || echo "install: $fails failed"
[ "$fails" -eq 0 ]
