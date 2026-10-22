#!/usr/bin/env bash
# Run every tests/*.test.sh; exit non-zero if any fails.
set -u
status=0
for t in "$(dirname "$0")"/*.test.sh; do
  bash "$t" || status=1
done
exit "$status"
