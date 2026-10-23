#!/usr/bin/env bash
# Release hygiene: the version in plugin.json is the release users get (an
# unchanged version means `claude plugin update` ships nothing), so every
# version must have a dated CHANGELOG.md entry, and the changelog must keep an
# Unreleased section for work in progress.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CHANGELOG="$ROOT/CHANGELOG.md"
fails=0
check() { # $1=description, rest=test command
  local d="$1"; shift
  if "$@"; then echo "ok   $d"; else echo "FAIL $d"; fails=$((fails + 1)); fi
}

version="$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1]))["version"])' "$ROOT/.claude-plugin/plugin.json" 2>/dev/null)"
check "plugin.json has a semver version" grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$' <<<"$version"
check "CHANGELOG.md exists" [ -f "$CHANGELOG" ]
check "CHANGELOG.md has an Unreleased section" grep -qxF '## [Unreleased]' "$CHANGELOG"
check "CHANGELOG.md has a dated entry for $version" grep -qE "^## \[${version//./\\.}\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" "$CHANGELOG"

[ "$fails" -eq 0 ] && echo "release: all passed" || echo "release: $fails failed"
[ "$fails" -eq 0 ]
