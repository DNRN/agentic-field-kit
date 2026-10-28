#!/usr/bin/env bash
# catalog.tsv is the installer's only list of components (ADR 0007): a component
# missing from it can't be installed, and a broken row breaks every install.
# Fails on a component on disk without a row, a row pointing at nothing, a name
# that differs from the component's frontmatter, and a bad or circular requires.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CATALOG="$ROOT/catalog.tsv"
fails=0
fail() { echo "FAIL $*"; fails=$((fails + 1)); }

[ "$(head -n1 "$CATALOG")" = "$(printf 'kind\tname\tpath\tharnesses\trequires\tsummary')" ] ||
  fail "catalog.tsv: header must be: kind name path harnesses requires summary (tab-separated)"

names="" paths=""
declare -a REQ_NAME REQ_LIST
while IFS= read -r line; do
  IFS=$'\t' read -r -a f <<<"$line"
  [ "${#f[@]}" -eq 6 ] || { fail "row needs 6 tab-separated columns: $line"; continue; }
  kind=${f[0]} name=${f[1]} path=${f[2]} harn=${f[3]} req=${f[4]}
  [[ "$name" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]] || fail "$name: name must be kebab-case"
  case " $names " in *" $name "*) fail "$name: listed twice" ;; esac
  names="$names $name" paths="$paths $path"
  REQ_NAME+=("$name") REQ_LIST+=("${req//,/ }")
  case "$kind" in skill|agent|hook) ;; *) fail "$name: kind must be skill, agent, or hook" ;; esac
  case "$harn" in all|claude|plugin) ;; *) fail "$name: harnesses must be all, claude, or plugin" ;; esac
  [ -e "$ROOT/$path" ] || { fail "$name: path $path does not exist"; continue; }
  case "$kind" in
    skill) grep -qx "name: $name" "$ROOT/$path/SKILL.md" || fail "$name: $path/SKILL.md frontmatter name differs" ;;
    agent) grep -qx "name: $name" "$ROOT/$path" || fail "$name: $path frontmatter name differs" ;;
    hook)  [ "$(basename "$path" .sh)" = "$name" ] || fail "$name: hook script must be named $name.sh" ;;
  esac
done < <(tail -n +2 "$CATALOG")

for i in "${!REQ_NAME[@]}"; do
  for r in ${REQ_LIST[i]}; do
    [ "$r" = - ] && continue
    case " $names " in *" $r "*) ;; *) fail "${REQ_NAME[i]}: requires unknown component $r" ;; esac
  done
done

# Follow requires from each component; reaching it again is a cycle.
reqs() {
  local i
  for i in "${!REQ_NAME[@]}"; do
    if [ "${REQ_NAME[i]}" = "$1" ] && [ "${REQ_LIST[i]}" != - ]; then printf '%s' "${REQ_LIST[i]}"; fi
  done
}
for start in $names; do
  seen="" todo=$(reqs "$start")
  while [ -n "${todo// /}" ]; do
    # shellcheck disable=SC2086 # split the work list into names
    set -- $todo
    n=$1; shift; todo="$*"
    if [ "$n" = "$start" ]; then fail "$start: requires itself through a cycle"; break; fi
    case " $seen " in *" $n "*) continue ;; esac
    seen="$seen $n" todo="$todo $(reqs "$n")"
  done
done

for path in "$ROOT"/skills/*/SKILL.md "$ROOT"/agents/*.md "$ROOT"/hooks/scripts/*.sh; do
  [ "$(basename "$path")" = README.md ] && continue
  rel=${path#"$ROOT/"}
  rel=${rel%/SKILL.md}
  case " $paths " in *" $rel "*) ;; *) fail "$rel has no row in catalog.tsv, so install.sh can't install it" ;; esac
done

[ "$fails" -eq 0 ] && echo "catalog: all passed" || echo "catalog: $fails failed"
[ "$fails" -eq 0 ]
