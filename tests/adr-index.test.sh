#!/usr/bin/env bash
# Regression tests for hooks/scripts/adr-index.sh (SessionStart: stdout is
# added to the session's context; silent when there is nothing to list).
set -u
HOOK="$(cd "$(dirname "$0")/.." && pwd)/hooks/scripts/adr-index.sh"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
fails=0
check() { # $1=description, rest=test command
  local d="$1"; shift
  if "$@"; then echo "ok   $d"; else echo "FAIL $d"; fails=$((fails + 1)); fi
}
run() { (cd "$WORK" && env -i PATH=/usr/bin:/bin CLAUDE_PROJECT_DIR="$1" bash "$HOOK" < /dev/null); }
adr() { # project file title status
  mkdir -p "$1/docs/adr"
  printf '# %s\n\nDate: 2026-10-30\n\n## Status\n\n%s\n\n## Context\n\nForces.\n' "$3" "$4" > "$1/docs/adr/$2"
}

P="$WORK/p"
adr "$P" 0001-record.md "0001. Record architecture decisions" Accepted
adr "$P" 0002-rest.md "0002. REST for service APIs" "Superseded by [0004](0004-grpc.md)"
adr "$P" 0003-queue.md "0003. A queue for jobs" Proposed
adr "$P" 0004-grpc.md "0004. gRPC for service APIs" "Accepted. Item 2 superseded by 0006."
adr "$P" 0005-cache.md "0005. No cache" "Rejected: not needed yet."
printf '# 0006. CRLF\r\n\r\n## Status\r\n\r\nAccepted\r\n' > "$P/docs/adr/0006-crlf.md"
printf '# 0007. Odd\n\n**Status:** Accepted\n' > "$P/docs/adr/0007-odd.md"
printf 'no heading\n\n## Status\n\nAccepted\n' > "$P/docs/adr/0008-untitled.md"
adr "$P" README.md "ADRs" Accepted
out="$(run "$P")"
expected="Accepted ADRs in docs/adr/ (read one before changing what it decides; supersede it rather than edit it):
0001 Record architecture decisions - docs/adr/0001-record.md
0004 gRPC for service APIs - docs/adr/0004-grpc.md
0006 CRLF - docs/adr/0006-crlf.md"
check "lists only Accepted ADRs, in order, one line each" [ "$out" = "$expected" ]
[ "$out" = "$expected" ] || printf '%s\n---\n' "$out"

mkdir -p "$WORK/none"
check "no docs/adr/: silent" [ -z "$(run "$WORK/none")" ]
adr "$WORK/proposed" 0001-a.md "0001. A" Proposed
check "no Accepted ADR: silent" [ -z "$(run "$WORK/proposed")" ]
check "exit 0 when silent" run "$WORK/none"

G="$WORK/git"
mkdir -p "$G/sub"; git -C "$G" init -q
adr "$G" 0001-a.md "0001. From git root" Accepted
out="$(cd "$G/sub" && env -i PATH=/usr/bin:/bin bash "$HOOK" < /dev/null)"
check "without CLAUDE_PROJECT_DIR: the git root of the working directory" [ "$(printf '%s\n' "$out" | tail -n1)" = "0001 From git root - docs/adr/0001-a.md" ]

M="$WORK/many"
for i in $(seq 1 60); do n="$(printf '%04d' "$i")"; adr "$M" "$n-d.md" "$n. Decision $i" Accepted; done
check "60 ADRs: 61 lines" [ "$(run "$M" | wc -l | tr -d ' ')" -eq 61 ]

[ "$fails" -eq 0 ] && echo "adr-index: all passed" || echo "adr-index: $fails failed"
[ "$fails" -eq 0 ]
