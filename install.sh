#!/usr/bin/env bash
# Install this repo's skills into agent harnesses' skill directories.
# Symlink by default: one `git pull` updates every harness at once.
#
# Verified locations (see README compatibility table):
#   ~/.agents/skills       read natively by Codex, Gemini CLI, Cursor, OpenCode
#   ~/.claude/skills       Claude Code (also read by Cursor and OpenCode)
#   ~/.omp/agent/skills    omp
set -euo pipefail
shopt -s nullglob

MODE=symlink
ACTION=install
HARNESSES=()

usage() {
  sed -n '2,12p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
  exit "${1:-0}"
}

die() { echo "install.sh: $*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --harness) [ $# -ge 2 ] || usage 1; HARNESSES+=("$2"); shift 2 ;;
    --mode)    [ $# -ge 2 ] || usage 1; MODE=$2; shift 2 ;;
    --remove)  ACTION=remove; shift ;;
    --check)   ACTION=check; shift ;;
    -h|--help) usage 0 ;;
    *) die "unknown argument: $1 (see --help)" ;;
  esac
done
[ "$MODE" = symlink ] || [ "$MODE" = copy ] || die "mode must be symlink or copy"

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILLS_SRC="$REPO/skills"
NAME_RE='^[a-z0-9]+(-[a-z0-9]+)*$'

target_dir() {
  case "$1" in
    agents) printf '%s' "$HOME/.agents/skills" ;;
    claude) printf '%s' "$HOME/.claude/skills" ;;
    omp)    printf '%s' "$HOME/.omp/agent/skills" ;;
    *) return 1 ;;
  esac
}

[ -d "$SKILLS_SRC" ] || die "no skills/ directory in $REPO"

# --check: validate every skill against the Agent Skills portability rules.
if [ "$ACTION" = check ]; then
  status=0
  for src in "$SKILLS_SRC"/*/; do
    name=$(basename "$src")
    [ -f "$src/SKILL.md" ] || { echo "FAIL $name: missing SKILL.md"; status=1; continue; }
    [[ "$name" =~ $NAME_RE ]] || { echo "FAIL $name: folder name must be lowercase alphanumeric with single hyphens"; status=1; }
    grep -q "^name: *$name\$" "$src/SKILL.md" || { echo "FAIL $name: frontmatter 'name: $name' missing or mismatched"; status=1; }
    grep -q '^description: *..*.' "$src/SKILL.md" || { echo "FAIL $name: empty or missing description"; status=1; }
  done
  [ "$status" -eq 0 ] && echo "all skills OK"
  exit "$status"
fi

mapfile -t ALL < <(compgen -W "agents claude omp")
if [ "${#HARNESSES[@]}" -eq 0 ]; then HARNESSES=("${ALL[@]}"); fi
for h in "${HARNESSES[@]}"; do target_dir "$h" >/dev/null || die "unknown harness: $h"; done

status=0
for src in "$SKILLS_SRC"/*/; do
  name=$(basename "$src")
  [ -f "$src/SKILL.md" ] || { echo "skip $name (no SKILL.md)"; continue; }
  for h in "${HARNESSES[@]}"; do
    dst="$(target_dir "$h")/$name"
    if [ "$ACTION" = remove ]; then
      if [ -L "$dst" ] && [ "$(readlink "$dst")" = "$src" ]; then
        rm "$dst"; echo "removed $dst"
      elif [ -f "$dst/.agentic-toolkit-install" ]; then
        rm -rf "$dst"; echo "removed $dst"
      fi
      continue
    fi
    if [ "$MODE" = symlink ]; then
      if [ -L "$dst" ] && [ "$(readlink "$dst")" = "$src" ]; then continue; fi
      if [ -e "$dst" ]; then
        if [ -L "$dst" ]; then rm "$dst"; else echo "skip $dst (real directory exists)"; status=1; continue; fi
      fi
      mkdir -p "$(dirname "$dst")"
      ln -s "$src" "$dst"; echo "linked   $dst -> $src"
    else
      rm -rf "$dst"
      mkdir -p "$(dirname "$dst")"
      cp -R "$src" "$dst"
      echo "$REPO" > "$dst/.agentic-toolkit-install"
      echo "copied   $dst"
    fi
  done
done
[ "$ACTION" = remove ] && exit 0
[ "$status" -eq 0 ] && echo "done" || echo "done with skips"
exit "$status"
