#!/usr/bin/env bash
# Prepare a release (README, Releasing, step 1), run from an up-to-date
# main: show the latest GitHub release, ask whether this one is major, minor, or
# patch, switch to a new release-X.Y.Z branch, move the Unreleased CHANGELOG.md
# entries under a dated heading for X.Y.Z, update the compare links, set X.Y.Z
# in .claude-plugin/plugin.json and installer/Cargo.toml, refresh Cargo.lock,
# and run tests/run.sh. Committing, the PR, and tagging stay manual.
#
# The suggested answer comes from Unreleased: a **Breaking** entry or a Removed
# section is major (minor while 0.x), Added, Changed, or Deprecated is minor,
# Fixed or Security only is patch. Pass the answer as an argument to skip the
# question. Needs gh (https://cli.github.com), signed in.
#
# Usage: scripts/prepare-release.sh [major|minor|patch]
# shellcheck disable=SC2016 # the single-quoted awk programs passed to rewrite
set -euo pipefail
cd "$(dirname "$0")/.."
CHANGELOG=CHANGELOG.md PLUGIN=.claude-plugin/plugin.json CARGO=installer/Cargo.toml
die() { echo "prepare-release: $*" >&2; exit 1; }

case "${1-}" in major | minor | patch | "") ;; *) die "usage: $0 [major|minor|patch]" ;; esac
[ $# -le 1 ] || die "usage: $0 [major|minor|patch]"
# The release branch is cut from main, which must match GitHub's.
[ "$(git branch --show-current)" = main ] || die "run from main; it creates the release-X.Y.Z branch itself"
git diff --quiet HEAD || die "uncommitted changes; commit or stash them first, or they'd move to the release branch"
git fetch -q origin main || die "can't fetch origin main"
git merge-base --is-ancestor origin/main HEAD || die "main is behind origin/main; git pull first"
plugin_field() { python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))[sys.argv[2]])' "$PLUGIN" "$1"; }
name="$(plugin_field name)" repo="$(plugin_field repository)" plugin_version="$(plugin_field version)"

# The latest GitHub release is built from the installer/vX.Y.Z tag (README, Releasing, step 4).
command -v gh >/dev/null || die "needs gh (https://cli.github.com) to read the latest release"
release="$(gh release view --repo "${repo#https://github.com/}" --json tagName,publishedAt,url \
  --jq '[.tagName, .publishedAt[:10], .url] | @tsv')" || die "can't read the latest release of $repo with gh; run gh auth login"
IFS=$'\t' read -r release_tag release_date release_url <<<"$release"
current="${release_tag#installer/v}"
grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$' <<<"$current" || die "latest release tag $release_tag isn't installer/vX.Y.Z"
[ "$plugin_version" = "$current" ] ||
  die "$PLUGIN has version $plugin_version, but the latest GitHub release is $current; publish $plugin_version first (README, Releasing, steps 3-4) or fix $PLUGIN"
changelog_version="$(awk -F'[][]' '/^## \[[0-9]+\.[0-9]+\.[0-9]+\] - / { print $2; exit }' "$CHANGELOG")"
[ "$changelog_version" = "$current" ] ||
  die "the newest release in $CHANGELOG is ${changelog_version:-missing}, but the latest GitHub release is $current"

# "SUGGESTION REASON SUMMARY", separated by \037 (unit separator, not
# whitespace, so an empty REASON keeps its place); SUGGESTION is major, minor,
# patch, unknown, or none.
result="$(awk '
  $0 == "## [Unreleased]" { in_unreleased = 1; next }
  /^## \[/ { in_unreleased = 0 }
  !in_unreleased { next }
  /^### / { section = substr($0, 5); next }
  /^- / {
    key = section == "" ? "no section" : section
    if (!(key in count)) order[n++] = key
    count[key]++
    if (/\*\*Breaking/) { l = 3; why = "a **Breaking** entry"; breaking++ }
    else if (section == "Removed") { l = 3; why = "### Removed" }
    else if (section == "Added" || section == "Changed" || section == "Deprecated") { l = 2; why = "### " section }
    else if (section == "Fixed" || section == "Security") { l = 1; why = "### " section }
    else { unknown = 1; l = 0 }
    if (l > level) { level = l; reason = why }
  }
  END {
    for (i = 0; i < n; i++) summary = summary (i ? ", " : "") order[i] " (" count[order[i]] ")"
    if (breaking) summary = summary ", " breaking " **Breaking**"
    print (n == 0 ? "none" : unknown ? "unknown" : level == 3 ? "major" : level == 2 ? "minor" : "patch") "\037" reason "\037" summary
  }
' "$CHANGELOG")"
IFS=$'\037' read -r suggestion reason summary <<<"$result"
[ "$suggestion" = none ] && die "nothing under ## [Unreleased] in $CHANGELOG to release"
IFS=. read -r major minor patch <<<"$current"
if [ "$suggestion" = major ] && [ "$major" -eq 0 ]; then suggestion=minor reason="$reason, minor while 0.x"; fi

echo "Latest GitHub release: $current ($release_tag, $release_date)"
echo "  $release_url"
echo "Unreleased in $CHANGELOG: $summary"
echo "  major -> $((major + 1)).0.0   minor -> $major.$((minor + 1)).0   patch -> $major.$minor.$((patch + 1))"
bump="${1-}"
if [ -z "$bump" ]; then
  [ -t 0 ] || die "no terminal to ask on; pass major, minor, or patch"
  default=""
  [ "$suggestion" = unknown ] || default="$suggestion"
  while :; do
    read -r -p "Is this release major, minor, or patch?${default:+ [$default, from $reason]} " bump || die "no answer"
    bump="$(tr '[:upper:]' '[:lower:]' <<<"${bump:-$default}")"
    case "$bump" in major | minor | patch) break ;; *) echo "  answer major, minor, or patch" ;; esac
  done
fi
case "$bump" in
  major) new="$((major + 1)).0.0" ;;
  minor) new="$major.$((minor + 1)).0" ;;
  patch) new="$major.$minor.$((patch + 1))" ;;
esac
tag="$name--v$new" prev_tag="$name--v$current"
git rev-parse -q --verify "refs/tags/$tag" >/dev/null && die "tag $tag already exists"
grep -qF "## [$new]" "$CHANGELOG" && die "$CHANGELOG already has a $new entry"
branch="release-$new"
git rev-parse -q --verify "refs/heads/$branch" >/dev/null && die "branch $branch already exists"
git ls-remote --exit-code --heads origin "$branch" >/dev/null && die "branch $branch already exists on origin"
git switch -q -c "$branch"
echo "prepare-release: on new branch $branch"

# Rewrite FILE with AWK_PROGRAM (given -v vars); the program exits non-zero if
# it didn't find what it had to change, and FILE is left as it was.
rewrite() { # file, awk args...
  local file="$1"; shift
  awk "$@" "$file" >"$file.tmp" || { rm -f "$file.tmp"; die "unexpected layout in $file"; }
  mv "$file.tmp" "$file"
}
# Replace the first line equal to OLD with NEW.
replace_line() { # file, old, new
  rewrite "$1" -v old="$2" -v new="$3" \
    '!done && $0 == old { print new; done = 1; next } { print } END { exit !done }'
}

rewrite "$CHANGELOG" -v new="$new" -v date="$(date +%Y-%m-%d)" -v url="$repo" -v tag="$tag" -v prev="$prev_tag" '
  $0 == "## [Unreleased]" { print; print ""; print "## [" new "] - " date; heading = 1; next }
  /^\[Unreleased\]: / {
    print "[Unreleased]: " url "/compare/" tag "...HEAD"
    print "[" new "]: " url "/compare/" prev "..." tag
    link = 1; next
  }
  { print }
  END { exit !(heading && link) }'
replace_line "$PLUGIN" "  \"version\": \"$current\"," "  \"version\": \"$new\","
replace_line "$CARGO" "version = \"$current\"" "version = \"$new\""
cargo update --workspace --offline --quiet

echo "prepare-release: $current -> $new ($bump); running tests/run.sh"
if ! tests/run.sh; then
  echo "prepare-release: tests failed on $branch. Fix them there, or undo with:" >&2
  echo "  git checkout -- $CHANGELOG $PLUGIN $CARGO Cargo.lock && git switch main && git branch -d $branch" >&2
  exit 1
fi
cat <<EOF

prepare-release: $new is ready on $branch. Review the diff, then:
  git commit -am "Release $new"
  git push -u origin $branch && gh pr create --fill
After the merge, on main: git tag $tag && git push origin $tag
                          git tag installer/v$new && git push origin installer/v$new
EOF
