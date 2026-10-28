#!/usr/bin/env bash
# Install agentic-toolkit components (catalog.tsv) into a project or globally.
#
#   ./install.sh                     on a terminal: interactive menu
#   ./install.sh --list              what is installed, in the project and globally
#   ./install.sh NAME... | --all     install components and what they require
#   ./install.sh --update [NAME...]  after `git pull` here: refresh what changed
#   ./install.sh --remove [NAME...]  remove what this installer installed
#   ./install.sh --check             validate skills (portability)
#
# Scope: the git root of the working directory (default), --project DIR, or
# --global. Project installs are copies, recorded in .agents/agentic-toolkit.tsv;
# commit them with the project:
#   .agents/skills/<name>       every harness except Claude Code
#   .claude/skills/<name>       Claude Code, unless the project enables the plugin
#   .claude/agents/<name>.md    Claude Code subagents, same condition
# Global installs (skills only) are symlinks into this clone; --mode copy for
# copies, the default on Windows. Global targets, --harness (repeatable):
#   agents  ~/.agents/skills     default; Codex, Gemini CLI, OpenCode, Cursor, omp
#   claude  ~/.claude/skills     Claude Code WITHOUT the plugin: a symlink to the
#                                plugin's skill folder hides its names
#   omp     ~/.omp/agent/skills  omp with its ~/.agents provider turned off
#
# Options: --force        also overwrite or remove locally modified copies
#          --dry-run      print the plan only
#          --yes          do not ask (also removes retired and unneeded components)
#          --interactive  open the menu without a terminal
set -euo pipefail
shopt -s nullglob

SELF="${BASH_SOURCE[0]}"
REPO="$(cd "$(dirname "$SELF")" && pwd)"
SKILLS_SRC="$REPO/skills"
CATALOG="$REPO/catalog.tsv"
NAME_RE='^[a-z0-9]+(-[a-z0-9]+)*$'
TAB=$'\t'

usage() {
  sed -n '2,/^[^#]/{/^#/s/^# \{0,1\}//p;}' "$SELF"
  exit "${1:-0}"
}

die() { echo "install.sh: $*" >&2; exit 1; }

ACTION=""
NAMES=""
ALL=0 GLOBAL=0 FORCE=0 DRY=0 YES=0 MENU=0
PROJECT_ARG="" MODE="" HARNESSES=""

set_action() {
  [ -z "$ACTION" ] || [ "$ACTION" = "$1" ] || die "choose one of --list, --update, --remove, --check"
  ACTION=$1
}

while [ $# -gt 0 ]; do
  case "$1" in
    --list)        set_action list ;;
    --update)      set_action update ;;
    --remove)      set_action remove ;;
    --check)       set_action check ;;
    --all)         ALL=1 ;;
    --global)      GLOBAL=1 ;;
    --project)     [ $# -ge 2 ] || usage 1; PROJECT_ARG=$2; shift ;;
    --harness)     [ $# -ge 2 ] || usage 1; HARNESSES="$HARNESSES $2"; shift ;;
    --mode)        [ $# -ge 2 ] || usage 1; MODE=$2; shift ;;
    --force)       FORCE=1 ;;
    --dry-run)     DRY=1 ;;
    --yes|-y)      YES=1 ;;
    --interactive) MENU=1 ;;
    -h|--help)     usage 0 ;;
    -*)            die "unknown option: $1 (see --help)" ;;
    *)             NAMES="$NAMES $1" ;;
  esac
  shift
done
NAMES="${NAMES# }"
HARNESSES="${HARNESSES# }"
for n in $NAMES; do [[ "$n" =~ $NAME_RE ]] || die "not a component name: $n (see --list)"; done

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

is_tty() { [ -t 0 ] && [ -t 1 ]; }

# ---- catalog (ADR 0007) ------------------------------------------------------

[ -f "$CATALOG" ] || die "no catalog.tsv in $REPO"
C_KIND=() C_NAME=() C_PATH=() C_HARN=() C_REQ=() C_SUM=()
NCAT=0
while IFS="$TAB" read -r k n p h r s; do
  case "$k" in ''|'#'*|kind) continue ;; esac
  C_KIND[NCAT]=$k C_NAME[NCAT]=$n C_PATH[NCAT]=$p C_HARN[NCAT]=$h C_REQ[NCAT]=$r C_SUM[NCAT]=$s
  NCAT=$((NCAT + 1))
done < "$CATALOG"

IDX=0
cat_idx() { # name → sets IDX; fails for a name not in the catalog
  local i=0
  while [ "$i" -lt "$NCAT" ]; do
    if [ "${C_NAME[i]}" = "$1" ]; then IDX=$i; return 0; fi
    i=$((i + 1))
  done
  return 1
}

reqs_of() { # name → space-separated requires
  cat_idx "$1" || return 0
  [ "${C_REQ[IDX]}" = - ] || printf '%s' "${C_REQ[IDX]//,/ }"
}

all_names() { local i=0; while [ "$i" -lt "$NCAT" ]; do printf '%s ' "${C_NAME[i]}"; i=$((i + 1)); done; }

has() { case " $1 " in *" $2 "*) return 0 ;; esac; return 1; } # list word

VERSION=$(sed -n 's/^[[:space:]]*"version":[[:space:]]*"\([^"]*\)".*/\1/p' "$REPO/.claude-plugin/plugin.json")

# ---- scope ------------------------------------------------------------------

[ "$GLOBAL" = 0 ] || [ -z "$PROJECT_ARG" ] || die "choose --project DIR or --global, not both"
if [ -n "$HARNESSES$MODE" ] && [ "$GLOBAL" = 0 ]; then
  die "--harness and --mode apply to --global only: project installs are copies (ADR 0006)"
fi
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) DEFAULT_MODE=copy ;; *) DEFAULT_MODE=symlink ;; esac
MODE_ARG=$MODE
MODE=${MODE:-$DEFAULT_MODE}
[ "$MODE" = symlink ] || [ "$MODE" = copy ] || die "mode must be symlink or copy"

global_rel() {
  case "$1" in
    agents) printf '.agents/skills' ;;
    claude) printf '.claude/skills' ;;
    omp)    printf '.omp/agent/skills' ;;
    *) return 1 ;;
  esac
}
for h in $HARNESSES; do global_rel "$h" >/dev/null || die "unknown harness: $h"; done

ROOT="" PROJECT_WHY=""
REPO_REAL=$(cd "$REPO" && pwd -P)
if [ -n "$PROJECT_ARG" ]; then
  [ -d "$PROJECT_ARG" ] || die "no such directory: $PROJECT_ARG"
  ROOT=$(cd "$PROJECT_ARG" && pwd -P)
elif top=$(git rev-parse --show-toplevel 2>/dev/null); then
  ROOT=$(cd "$top" && pwd -P)
else
  PROJECT_WHY="not inside a git repository: run install.sh from your project, or use --project DIR or --global"
fi
if [ -n "$ROOT" ] && [ "$ROOT" = "$REPO_REAL" ]; then
  ROOT=""
  PROJECT_WHY="this is the agentic-toolkit clone, not a project: run install.sh from your project, or use --global"
elif [ -n "$ROOT" ] && [ "$ROOT" = "$(cd "$HOME" && pwd -P)" ]; then
  ROOT=""
  PROJECT_WHY="the project would be your home directory, which is the global scope: use --global, or --project DIR"
fi

scope_root() { if [ "$1" = global ]; then printf '%s' "$HOME"; else printf '%s' "$ROOT"; fi; }

disp() { # scope relpath → path as shown to the user
  # shellcheck disable=SC2088 # a literal ~ for display
  if [ "$1" = global ]; then printf '~/%s' "$2"; else printf '%s' "$2"; fi
}

plugin_on() { # root → is agentic-toolkit enabled in root/.claude/settings.json
  [ -f "$1/.claude/settings.json" ] &&
    grep -Eq '"agentic-toolkit@[^"]*"[[:space:]]*:[[:space:]]*true' "$1/.claude/settings.json"
}

# ---- hashing (ADR 0008): content only, carriage returns ignored -------------

if command -v sha256sum >/dev/null 2>&1; then
  sha() { sha256sum | cut -d' ' -f1; }
elif command -v shasum >/dev/null 2>&1; then
  sha() { shasum -a 256 | cut -d' ' -f1; }
else
  die "needs sha256sum or shasum"
fi

hash_of() { # file or folder → sha256 of its content
  if [ -f "$1" ]; then tr -d '\r' < "$1" | sha; return; fi
  (
    cd "$1" &&
      find . -type f ! -name .agentic-toolkit-install | LC_ALL=C sort |
      while IFS= read -r f; do printf '%s %s\n' "$(tr -d '\r' < "$f" | sha)" "$f"; done
  ) | sha
}

src_hash() { cat_idx "$1" && hash_of "$REPO/${C_PATH[IDX]}"; }

# ---- manifest (ADR 0008) ----------------------------------------------------

man_file() { printf '%s/.agents/agentic-toolkit.tsv' "$1"; }

# Rows are read with any trailing \r removed: the project manifest is committed,
# and a Windows checkout may convert it to CRLF.
man_row() { # root name → the name's row; fails when absent
  local f; f=$(man_file "$1")
  [ -f "$f" ] || return 1
  awk -F'\t' -v n="$2" '{ sub(/\r$/, "") } $1 != "kind" && $2 == n { print; found = 1 } END { exit !found }' "$f"
}

man_field() { cut -f"$2" <<<"$1"; } # row column

# Paths from a row, limited to those the installer writes for that name: a
# committed manifest must not steer rm -rf outside the component's folders.
row_paths() { # row → paths
  local n p out=""
  n=$(man_field "$1" 2)
  for p in $(man_field "$1" 6); do
    case "$p" in
      ".agents/skills/$n"|".claude/skills/$n"|".omp/agent/skills/$n"|".claude/agents/$n.md") out="$out $p" ;;
    esac
  done
  printf '%s' "${out# }"
}

reason_of() { # root name → manifest reason; explicit for a component not in it
  local row
  if row=$(man_row "$1" "$2"); then man_field "$row" 5; else echo explicit; fi
}

man_names() { # root → names in the manifest
  local f; f=$(man_file "$1")
  [ -f "$f" ] || return 0
  awk -F'\t' '{ sub(/\r$/, "") } $1 != "kind" && $2 ~ /^[a-z0-9]+(-[a-z0-9]+)*$/ { printf "%s ", $2 }' "$f"
}

man_put() { # root kind name hash reason paths; replaces the name's row
  local f rest
  f=$(man_file "$1")
  mkdir -p "$(dirname "$f")"
  rest=""
  [ -f "$f" ] && rest=$(awk -F'\t' -v n="$3" '{ sub(/\r$/, "") } $1 != "kind" && NF && $2 != n' "$f")
  {
    printf 'kind\tname\tversion\thash\treason\tpaths\n'
    { [ -z "$rest" ] || printf '%s\n' "$rest"; printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$2" "$3" "$VERSION" "$4" "$5" "$6"; } |
      LC_ALL=C sort -t"$TAB" -k2,2
  } > "$f.tmp"
  mv "$f.tmp" "$f"
}

man_drop() { # root name; deletes the manifest when it empties
  local f rest
  f=$(man_file "$1")
  [ -f "$f" ] || return 0
  rest=$(awk -F'\t' -v n="$2" '{ sub(/\r$/, "") } $1 != "kind" && NF && $2 != n' "$f")
  if [ -z "$rest" ]; then rm -f "$f"; return; fi
  { printf 'kind\tname\tversion\thash\treason\tpaths\n'; printf '%s\n' "$rest"; } > "$f.tmp"
  mv "$f.tmp" "$f"
}

# ---- what is where ----------------------------------------------------------

candidates() { # scope name → every path the component may occupy
  cat_idx "$2" || return 0
  case "$1:${C_KIND[IDX]}" in
    project:skill) printf '.agents/skills/%s .claude/skills/%s' "$2" "$2" ;;
    project:agent) printf '.claude/agents/%s.md' "$2" ;;
    global:skill)  printf '%s/%s .claude/skills/%s .omp/agent/skills/%s' "$(global_rel agents)" "$2" "$2" "$2" ;;
  esac
}

layout() { # scope name → paths a fresh install writes
  cat_idx "$2" || return 0
  case "$1:${C_KIND[IDX]}" in
    project:skill)
      printf '.agents/skills/%s' "$2"
      plugin_on "$ROOT" || printf ' .claude/skills/%s' "$2" ;;
    project:agent) printf '.claude/agents/%s.md' "$2" ;;
    global:skill)
      local h sep=""
      for h in ${HARNESSES:-agents}; do printf '%s%s/%s' "$sep" "$(global_rel "$h")" "$2"; sep=" "; done ;;
  esac
}

avail() { # scope name → empty when installable here (or unknown), else plugin or n/a
  cat_idx "$2" || { echo ""; return; }
  case "$1:${C_KIND[IDX]}" in
    project:skill|global:skill) echo "" ;;
    project:agent) if plugin_on "$ROOT"; then echo plugin; else echo ""; fi ;;
    *) case "${C_HARN[IDX]}" in
         claude|plugin) if plugin_on "$(scope_root "$1")"; then echo plugin; else echo n/a; fi ;;
         *) echo n/a ;;
       esac ;;
  esac
}

# Installed before the manifest existed: a symlink into this clone's skills/,
# or a copy carrying the old marker file.
legacy_ours() {
  if [ -L "$1" ]; then
    case "$(readlink "$1")" in "$SKILLS_SRC"/*) return 0 ;; esac
    return 1
  fi
  [ -f "$1/.agentic-toolkit-install" ]
}

exists() { [ -L "$1" ] || [ -e "$1" ]; }

owned() { # scope name → paths that are ours: manifest paths, then legacy ones
  local root row paths="" p
  root=$(scope_root "$1")
  if row=$(man_row "$root" "$2"); then paths=$(row_paths "$row"); fi
  for p in $(candidates "$1" "$2"); do
    has "$paths" "$p" && continue
    exists "$root/$p" && legacy_ours "$root/$p" && paths="$paths $p"
  done
  printf '%s' "${paths# }"
}

status() { # scope name → one status word (plan "Status values")
  local scope=$1 name=$2 root row mhash paths p full legacy=0 n=0 links=0 missing=0 dangling=0 modified=0
  root=$(scope_root "$scope")
  # A row whose paths are all rejected by row_paths claims nothing: treat it as absent.
  if row=$(man_row "$root" "$name") && [ -n "$(row_paths "$row")" ]; then
    mhash=$(man_field "$row" 4)
    paths=$(row_paths "$row")
  else
    paths=$(owned "$scope" "$name")
    if [ -z "$paths" ]; then
      for p in $(candidates "$scope" "$name"); do
        if exists "$root/$p"; then echo foreign; return; fi
      done
      p=$(avail "$scope" "$name")
      echo "${p:--}"
      return
    fi
    legacy=1
    mhash=$(src_hash "$name")
  fi
  for p in $paths; do
    full="$root/$p"
    n=$((n + 1))
    if [ -L "$full" ]; then
      links=$((links + 1))
      [ -e "$full" ] || dangling=1
    elif [ ! -e "$full" ]; then
      missing=1
    elif [ "$(hash_of "$full")" != "$mhash" ]; then
      modified=1
    fi
  done
  # An edit anywhere wins: nothing may overwrite or delete it without --force.
  if [ "$modified" = 1 ]; then echo modified
  elif ! cat_idx "$name"; then echo retired
  elif [ "$missing" = 1 ]; then echo missing
  elif [ "$dangling" = 1 ]; then echo dangling
  elif [ "$links" = "$n" ]; then echo linked
  elif [ "$legacy" = 1 ] || [ "$(src_hash "$name")" = "$mhash" ]; then echo installed
  else echo outdated
  fi
}

is_ours_status() { case "$1" in installed|linked|outdated|modified|missing|dangling|retired) return 0 ;; esac; return 1; }

# ---- plan -------------------------------------------------------------------
# One line per step: verb <TAB> name <TAB> reason <TAB> paths <TAB> note.
# verb: write (copy or link), remove, promote (reason → explicit), skip, ask-remove
# (remove only after a confirmation or --yes).

PLAN=""
RC=0
add() { PLAN="$PLAN$1$TAB$2$TAB$3$TAB$4$TAB$5"$'\n'; }
skip() { add skip "$1" - - "$2"; RC=1; }

# Global installs made as symlinks are recorded with hash "-" (a link has no
# content of its own), which is how a deleted one is restored as a link again.
form_for() { # scope full name → link or copy: --mode if given, else the form the path has, else the recorded one
  local row
  if [ "$1" = project ]; then echo copy
  elif [ -n "$MODE_ARG" ]; then if [ "$MODE" = symlink ]; then echo link; else echo copy; fi
  elif [ -L "$2" ]; then echo link
  elif exists "$2"; then echo copy
  elif row=$(man_row "$HOME" "$3"); then if [ "$(man_field "$row" 4)" = - ]; then echo link; else echo copy; fi
  elif [ "$MODE" = symlink ]; then echo link
  else echo copy
  fi
}

form_ok() { # scope full name → the path already has the form it would be written in
  if [ -L "$2" ]; then [ "$(form_for "$1" "$2" "$3")" = link ]; else [ "$(form_for "$1" "$2" "$3")" = copy ]; fi
}

union() { local out=$1 p; for p in $2; do has "$out" "$p" || out="$out $p"; done; printf '%s' "${out# }"; }

ORDER="" VISITING=""
visit() { # name → appends it to ORDER after everything it requires
  local n=$1 r rs
  has "$ORDER" "$n" && return 0
  has "$VISITING" "$n" && die "dependency cycle at $n"
  cat_idx "$n" || die "unknown component: $n (see --list)"
  VISITING="$VISITING $n"
  rs=$(reqs_of "$n")
  for r in $rs; do visit "$r"; done
  ORDER="$ORDER $n"
}

plan_write() { # scope name reason explicit(0|1)
  local scope=$1 n=$2 reason=$3 explicit=$4 root st av lay own row p need=0 mreason=""
  root=$(scope_root "$scope")
  av=$(avail "$scope" "$n")
  if [ -n "$av" ]; then
    case "$av" in
      plugin) skip "$n" "provided by the agentic-toolkit plugin enabled for this scope" ;;
      *) cat_idx "$n"
         if [ "${C_HARN[IDX]}" = plugin ]; then skip "$n" "plugin only until roadmap phase 1 (portable commit gate)"
         else skip "$n" "not available in this scope"; fi ;;
    esac
    return
  fi
  st=$(status "$scope" "$n")
  lay=$(layout "$scope" "$n")
  own=$(owned "$scope" "$n")
  for p in $lay; do
    if exists "$root/$p" && ! has "$own" "$p"; then
      skip "$n" "$(disp "$scope" "$p") exists, not installed by agentic-toolkit"
      return
    fi
  done
  if row=$(man_row "$root" "$n"); then mreason=$(man_field "$row" 5); fi
  [ "$explicit" = 1 ] && reason=explicit
  [ -n "$mreason" ] && [ "$reason" != explicit ] && reason=$mreason
  case "$st" in
    modified)
      if [ "$FORCE" = 0 ]; then skip "$n" "modified locally; --force overwrites it"; return; fi
      need=1 ;;
    installed|linked)
      for p in $lay; do
        if ! has "$(row_paths "${row:-}")" "$p" || ! form_ok "$scope" "$root/$p" "$n"; then need=1; fi
      done ;;
    *) need=1 ;;
  esac
  if [ "$need" = 1 ]; then
    add write "$n" "$reason" "$(union "$own" "$lay")" "$st"
  elif [ "$reason" = explicit ] && [ "$mreason" = dependency ]; then
    add promote "$n" explicit - ""
  else
    add ok "$n" - - "already installed"
  fi
}

plan_install() { # scope
  local scope=$1 n names
  if [ "$ALL" = 1 ]; then
    names=""
    for n in $(all_names); do [ -n "$(avail "$scope" "$n")" ] || names="$names $n"; done
  else
    names=$NAMES
  fi
  ORDER="" VISITING=""
  for n in $names; do
    cat_idx "$n" || die "unknown component: $n (see --list)"
    # Not installable here: say why, and don't pull in what it requires.
    if [ -n "$(avail "$scope" "$n")" ]; then plan_write "$scope" "$n" explicit 1; else visit "$n"; fi
  done
  for n in $ORDER; do
    if has "$names" "$n"; then plan_write "$scope" "$n" explicit 1; else plan_write "$scope" "$n" dependency 0; fi
  done
}

plan_update() { # scope
  local scope=$1 root n targets st r rs row
  root=$(scope_root "$scope")
  if [ -n "$NAMES" ]; then
    targets=$NAMES
  else
    targets=$(man_names "$root")
    for n in $(all_names); do
      has "$targets" "$n" || [ -z "$(owned "$scope" "$n")" ] || targets="$targets $n"
    done
  fi
  for n in $targets; do
    st=$(status "$scope" "$n")
    case "$st" in
      installed|linked) add ok "$n" - - "up to date" ;;
      outdated|missing|dangling) add write "$n" "$(reason_of "$root" "$n")" "$(owned "$scope" "$n")" "$st" ;;
      modified)
        if ! cat_idx "$n"; then
          if [ "$FORCE" = 1 ]; then
            add ask-remove "$n" - "$(owned "$scope" "$n")" "retired: no longer in the toolkit"
          else
            skip "$n" "retired, and modified locally; --force removes it"
          fi
        elif [ "$FORCE" = 1 ]; then
          add write "$n" "$(reason_of "$root" "$n")" "$(owned "$scope" "$n")" "$st"
        else
          skip "$n" "modified locally; --force overwrites it"
        fi ;;
      retired) add ask-remove "$n" - "$(owned "$scope" "$n")" "retired: no longer in the toolkit" ;;
      *) skip "$n" "not installed" ;;
    esac
  done
  # Requirements added to the toolkit since these components were installed.
  ORDER="" VISITING=""
  for n in $targets; do
    cat_idx "$n" || continue
    rs=$(reqs_of "$n")
    for r in $rs; do
      st=$(status "$scope" "$r")
      [ "$st" = - ] && visit "$r"
    done
  done
  for n in $ORDER; do plan_write "$scope" "$n" dependency 0; done
}

plan_remove() { # scope
  local scope=$1 root n m r targets="" kept="" edited="" auto="" changed st paths installed req_by
  root=$(scope_root "$scope")
  if [ -n "$NAMES" ]; then
    for n in $NAMES; do
      st=$(status "$scope" "$n")
      is_ours_status "$st" || { skip "$n" "not installed by agentic-toolkit here"; continue; }
      targets="$targets $n"
    done
  else
    targets=$(man_names "$root")
    for n in $(all_names); do
      has "$targets" "$n" || [ -z "$(owned "$scope" "$n")" ] || targets="$targets $n"
    done
  fi
  for n in $targets; do
    if [ "$(status "$scope" "$n")" = modified ] && [ "$FORCE" = 0 ]; then
      skip "$n" "modified locally; --force removes it"
      edited="$edited $n"
    else
      kept="$kept $n"
    fi
  done
  targets=$kept
  installed=$(man_names "$root")
  for m in $installed; do
    has "$targets" "$m" && continue
    for r in $(reqs_of "$m"); do
      has "$targets" "$r" || continue
      has "$edited" "$m" || die "$r is required by $m; remove both: install.sh --remove $r $m"
      skip "$r" "required by $m, which is modified locally; --force removes both"
      kept=""
      for n in $targets; do [ "$n" = "$r" ] || kept="$kept $n"; done
      targets=$kept
    done
  done
  # Dependencies nobody needs any more, pulled in only for what is removed.
  changed=1
  while [ "$changed" = 1 ]; do
    changed=0
    for m in $installed; do
      has "$targets" "$m" && continue
      [ "$(man_field "$(man_row "$root" "$m")" 5)" = dependency ] || continue
      req_by=""
      for n in $installed; do
        has "$targets" "$n" && continue
        has "$(reqs_of "$n")" "$m" && req_by="$req_by $n"
      done
      [ -z "$req_by" ] || continue
      for n in $targets; do
        if has "$(reqs_of "$n")" "$m"; then
          [ "$(status "$scope" "$m")" = modified ] && [ "$FORCE" = 0 ] && break
          targets="$targets $m"
          auto="$auto $m"
          add ask-remove "$m" - "$(owned "$scope" "$m")" "no longer needed"
          changed=1
          break
        fi
      done
    done
  done
  for n in $targets; do
    has "$auto" "$n" && continue
    paths=$(owned "$scope" "$n")
    if [ "$scope" = global ] && [ -n "$HARNESSES" ]; then paths=$(only_harnesses "$paths"); fi
    [ -n "$paths" ] && add remove "$n" - "$paths" ""
  done
  return 0
}

only_harnesses() { # paths → those under a --harness target
  local p h out=""
  for p in $1; do
    for h in $HARNESSES; do
      case "$p" in "$(global_rel "$h")"/*) out="$out $p" ;; esac
    done
  done
  printf '%s' "${out# }"
}

# ---- show and apply ----------------------------------------------------------

CONFIRMED=0
can_ask_remove() { [ "$YES" = 1 ] || [ "$CONFIRMED" = 1 ] || { is_tty && [ "$DRY" = 0 ]; }; }

show_plan() { # scope
  local scope=$1 verb n reason paths note p how root
  root=$(scope_root "$scope")
  while IFS="$TAB" read -r verb n reason paths note; do
    [ -n "$verb" ] || continue
    case "$verb" in
      write)
        case "$note" in outdated|modified|missing|dangling) how=update ;; *) how="" ;; esac
        for p in $paths; do
          printf '  %-7s %s%s\n' "${how:-$(form_for "$scope" "$root/$p" "$n")}" "$(disp "$scope" "$p")" "$([ "$reason" = dependency ] && printf '  (required)')"
        done ;;
      remove|ask-remove)
        if [ "$verb" = ask-remove ] && ! can_ask_remove; then
          printf '  %-7s %s (%s): remove it with --remove %s\n' keep "$n" "$note" "$n"
        else
          for p in $paths; do printf '  %-7s %s%s\n' remove "$(disp "$scope" "$p")" "${note:+  ($note)}"; done
        fi ;;
      promote) printf '  %-7s %s now counts as installed by you, not as a requirement\n' keep "$n" ;;
      skip|ok) printf '  %-7s %s: %s\n' "$verb" "$n" "$note" ;;
    esac
  done <<<"$PLAN"
}

plan_changes() { grep -Eq "^(write|remove|ask-remove|promote)$TAB" <<<"$PLAN"; }

tidy() { # project root: drop folders the removals emptied
  local d
  for d in .agents/skills .agents .claude/skills .claude/agents .claude; do
    rmdir "$1/$d" 2>/dev/null || true
  done
}

apply_plan() { # scope
  local scope=$1 root verb n reason paths note p full src h row left
  root=$(scope_root "$scope")
  while IFS="$TAB" read -r verb n reason paths note; do
    case "$verb" in
      write)
        if ! cat_idx "$n"; then echo "  skip    $n: not in the catalog any more"; RC=1; continue; fi
        src="$REPO/${C_PATH[IDX]}"
        h=-
        for p in $paths; do
          [ "$(form_for "$scope" "$root/$p" "$n")" = link ] || { h=$(hash_of "$src"); break; }
        done
        for p in $paths; do
          full="$root/$p"
          if [ "$(form_for "$scope" "$full" "$n")" = link ]; then rm -rf "$full"; mkdir -p "$(dirname "$full")"; ln -s "$src" "$full"
          else rm -rf "$full"; mkdir -p "$(dirname "$full")"; cp -R "$src" "$full"; fi
        done
        man_put "$root" "${C_KIND[IDX]}" "$n" "$h" "$reason" "$paths"
        WROTE="$WROTE $n" ;;
      remove|ask-remove)
        [ "$verb" = remove ] || can_ask_remove || continue
        for p in $paths; do rm -rf "${root:?}/$p"; done
        if row=$(man_row "$root" "$n"); then
          left=""
          for p in $(row_paths "$row"); do has "$paths" "$p" || left="$left $p"; done
          if [ -n "$left" ]; then
            man_put "$root" "$(man_field "$row" 1)" "$n" "$(man_field "$row" 4)" "$(man_field "$row" 5)" "${left# }"
          else
            man_drop "$root" "$n"
          fi
        fi
        CHANGED=1 ;;
      promote)
        row=$(man_row "$root" "$n")
        man_put "$root" "$(man_field "$row" 1)" "$n" "$(man_field "$row" 4)" explicit "$(row_paths "$row")" ;;
    esac
  done <<<"$PLAN"
  [ "$scope" = project ] && tidy "$root"
  return 0
}

confirm() { # question → yes?
  local ans
  printf '%s [y/N] ' "$1"
  IFS= read -r ans || ans=""
  case "$ans" in y|Y|yes|YES) return 0 ;; esac
  return 1
}

# Same skill in a project and globally: Codex lists it twice; Claude Code and
# Cursor use the global copy (plan "Evidence").
warn_both_scopes() {
  local n g
  while IFS= read -r n <&3; do
    cat_idx "$n" && [ "${C_KIND[IDX]}" = skill ] || continue
    g=$(status global "$n")
    case "$g" in -|n/a|plugin) continue ;; esac
    echo "! $n is also installed globally ($g): Codex lists it twice; Claude Code and Cursor use the global copy."
    if [ "$g" != foreign ] && is_tty && [ "$YES" = 0 ] && confirm "  Remove the global copy?"; then
      "$SELF" --global --remove "$n" --yes || RC=1
    elif [ "$g" != foreign ]; then
      echo "  Remove it with: $SELF --global --remove $n"
    fi
  done 3< <(awk -F'\t' '$1 == "write" || $1 == "ok" || $1 == "promote" { print $2 }' <<<"$PLAN")
}

run_action() { # action scope
  local action=$1 scope=$2
  PLAN="" WROTE="" CHANGED=0
  case "$action" in
    install) plan_install "$scope" ;;
    update)  plan_update "$scope" ;;
    remove)  plan_remove "$scope" ;;
  esac
  if [ -z "$PLAN" ]; then echo "Nothing to do."; return; fi
  show_plan "$scope"
  [ "$DRY" = 0 ] || return 0
  if plan_changes; then
    if is_tty && [ "$YES" = 0 ]; then
      if confirm "Apply?"; then CONFIRMED=1; else echo "Nothing changed."; return 0; fi
    fi
    apply_plan "$scope"
  fi
  if [ "$scope" = project ]; then
    [ "$action" != install ] || warn_both_scopes
    if [ -n "$WROTE" ] || [ "$CHANGED" = 1 ]; then
      echo "Commit the changes so teammates get the same set: git -C \"$ROOT\" add .agents .claude"
    fi
  fi
  plan_changes || return 0
  [ "$RC" = 0 ] && echo "done" || echo "done with skips"
}

# ---- list and menu -------------------------------------------------------------

detected_harnesses() {
  local out="" spec cli dir
  for spec in claude:.claude codex:.codex gemini:.gemini opencode:.config/opencode omp:.omp cursor-agent:.cursor; do
    cli=${spec%%:*} dir=${spec#*:}
    if command -v "$cli" >/dev/null 2>&1 || [ -d "$HOME/$dir" ]; then out="$out, ${cli%-agent}"; fi
  done
  printf '%s' "${out#, }"
}

ROWS=""
rows() { # catalog names, then retired names from either manifest
  local n
  ROWS=$(all_names)
  for n in $( [ -z "$ROOT" ] || man_names "$ROOT"; man_names "$HOME"); do
    has "$ROWS" "$n" || ROWS="$ROWS $n"
  done
}

MARKS=""
mark_of() { local m; for m in $MARKS; do case "$m" in "$1="*) printf '%s' "${m#*=}"; return ;; esac; done; printf ' '; }
set_mark() { # name mark ("" clears)
  local m out=""
  for m in $MARKS; do case "$m" in "$1="*) ;; *) out="$out $m" ;; esac; done
  [ -z "$2" ] || out="$out $1=$2"
  MARKS=${out# }
}

table() { # marks(0|1)
  local i=0 n kind st_p st_g sum notes row plug="not enabled" dup=0
  if [ -n "$ROOT" ]; then
    echo "agentic-toolkit $VERSION → project $ROOT"
    plugin_on "$ROOT" && plug="enabled for this project"
  else
    echo "agentic-toolkit $VERSION → no project (${PROJECT_WHY%%:*})"
  fi
  [ "$plug" != "not enabled" ] || ! plugin_on "$HOME" || plug="enabled for your user"
  echo "Harnesses found: $(detected_harnesses)"
  echo "Claude Code plugin: $plug"
  echo
  printf '  %2s  %-14s %-6s %-9s %-9s %s\n' "#" component kind project global summary
  rows
  for n in $ROWS; do
    i=$((i + 1))
    notes="" sum=""
    if cat_idx "$n"; then
      kind=${C_KIND[IDX]} sum=${C_SUM[IDX]}
      case "${C_HARN[IDX]}" in claude) notes="Claude Code only" ;; plugin) notes="plugin only until roadmap phase 1" ;; esac
      [ "${C_REQ[IDX]}" = - ] || notes="${notes:+$notes; }needs ${C_REQ[IDX]//,/, }"
    else
      row=$( { [ -n "$ROOT" ] && man_row "$ROOT" "$n"; } || man_row "$HOME" "$n" || true)
      kind=$(man_field "$row" 1)
      notes="no longer in the toolkit"
    fi
    if [ -n "$ROOT" ]; then st_p=$(status project "$n"); else st_p=n/a; fi
    st_g=$(status global "$n")
    if is_ours_status "$st_p" && case "$st_g" in -|n/a|plugin) false ;; *) true ;; esac; then
      notes="${notes:+$notes; }! in both scopes"
      dup=1
    fi
    [ "${#sum}" -le 31 ] || sum="${sum:0:30}…"
    printf '%s %2d  %-14s %-6s %-9s %-9s %s\n' "$([ "$1" = 1 ] && mark_of "$n" || printf ' ')" "$i" "$n" "$kind" "$st_p" "$st_g" "$sum"
    [ -z "$notes" ] || printf '      %s\n' "$notes"
  done
  echo
  [ "$dup" = 0 ] || echo "! in both scopes: Codex lists it twice; Claude Code and Cursor use the global copy."
  return 0
}

menu() {
  local scope=project line tok n h i st av adds rems ups rc row
  local -a sargs
  if [ -z "$ROOT" ]; then
    echo "$PROJECT_WHY"
    echo "Showing the global scope."
    scope=global
  fi
  [ "$GLOBAL" = 0 ] || scope=global
  while :; do
    table 1
    echo "Scope: $scope$([ "$scope" = project ] && printf ' (%s)' "$ROOT")"
    echo "Number: mark + install / - remove · u: mark updates (~) · g: other scope"
    echo "Enter: review and apply · q: quit"
    printf '> '
    IFS= read -r line || { echo; echo "No changes."; return 0; }
    case "$line" in
      q|Q) echo "No changes."; return 0 ;;
      g|G)
        if [ "$scope" = global ]; then
          if [ -z "$ROOT" ]; then echo "$PROJECT_WHY"; else scope=project; fi
        else
          scope=global
        fi
        MARKS=""
        continue ;;
      u|U)
        i=0
        for n in $ROWS; do
          case "$(status "$scope" "$n")" in outdated|missing|dangling|retired) set_mark "$n" "~"; i=1 ;; esac
        done
        [ "$i" = 1 ] || echo "Nothing to update."
        continue ;;
      "")
        adds="" rems="" ups=""
        for n in $ROWS; do
          case "$(mark_of "$n")" in "+") adds="$adds $n" ;; "-") rems="$rems $n" ;; "~") ups="$ups $n" ;; esac
        done
        if [ -z "$adds$rems$ups" ]; then echo "Nothing marked."; continue; fi
        if [ "$scope" = project ]; then
          sargs=(--project "$ROOT")
        else
          sargs=(--global)
          for h in $HARNESSES; do sargs+=(--harness "$h"); done
          [ -z "$MODE_ARG" ] || sargs+=(--mode "$MODE_ARG")
        fi
        [ "$FORCE" = 0 ] || sargs+=(--force)
        echo
        # The review runs with --yes too, so it shows exactly what applying does.
        # shellcheck disable=SC2086 # names split on purpose
        {
          [ -z "$adds" ] || "$SELF" "${sargs[@]}" $adds --dry-run --yes
          [ -z "$ups" ] || "$SELF" "${sargs[@]}" --update $ups --dry-run --yes
          [ -z "$rems" ] || "$SELF" "${sargs[@]}" --remove $rems --dry-run --yes
        } | grep -v '^Nothing to do\.$' || true
        if [ "$DRY" = 1 ]; then echo "Dry run: nothing changed."; return 0; fi
        if confirm "Apply?"; then
          rc=0
          # shellcheck disable=SC2086
          {
            [ -z "$adds" ] || "$SELF" "${sargs[@]}" $adds --yes || rc=1
            [ -z "$ups" ] || "$SELF" "${sargs[@]}" --update $ups --yes || rc=1
            [ -z "$rems" ] || "$SELF" "${sargs[@]}" --remove $rems --yes || rc=1
          }
          return "$rc"
        fi
        echo "Nothing changed."
        continue ;;
    esac
    for tok in $line; do
      case "$tok" in ''|*[!0-9]*|0*) echo "Not a component number: $tok"; continue ;; esac
      # shellcheck disable=SC2086 # one name per line
      n=$(printf '%s\n' $ROWS | sed -n "${tok}p")
      if [ -z "$n" ]; then echo "No component $tok"; continue; fi
      if [ "$(mark_of "$n")" != " " ]; then set_mark "$n" ""; continue; fi
      st=$(status "$scope" "$n")
      av=$(avail "$scope" "$n")
      case "$st" in
        -) set_mark "$n" "+" ;;
        foreign) echo "$n: exists but was not installed by agentic-toolkit; left alone" ;;
        plugin) echo "$n: provided by the agentic-toolkit plugin" ;;
        n/a)
          cat_idx "$n"
          if [ "${C_HARN[IDX]}" = plugin ]; then echo "$n: plugin only until roadmap phase 1 (portable commit gate)"
          else echo "$n: not available in the $scope scope${av:+ ($av)}"; fi ;;
        *) set_mark "$n" "-" ;;
      esac
    done
  done
}

# ---- main ---------------------------------------------------------------------

if [ -z "$ACTION" ]; then
  if [ -n "$NAMES" ] || [ "$ALL" = 1 ]; then
    ACTION=install
  elif [ "$MENU" = 1 ] || is_tty; then
    ACTION=menu
  else
    die "name the components to install, or use --all, --list, --update, --remove (see --help). Components: $(all_names)"
  fi
fi
[ "$ALL" = 0 ] || [ -z "$NAMES" ] || die "give component names or --all, not both"
[ "$ALL" = 0 ] || [ "$ACTION" = install ] || die "--all installs; --update and --remove without names cover everything"
case "$ACTION" in list|menu) [ -z "$NAMES" ] || die "--list and the menu take no component names" ;; esac

case "$ACTION" in
  list) table 0 ;;
  menu) menu ;;
  *)
    if [ "$GLOBAL" = 1 ]; then
      SCOPE=global
    else
      [ -n "$ROOT" ] || die "$PROJECT_WHY"
      SCOPE=project
    fi
    run_action "$ACTION" "$SCOPE"
    exit "$RC" ;;
esac
