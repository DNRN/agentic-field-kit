#!/usr/bin/env bash
set -euo pipefail
# Code change with tracked docs left stale. The review stamp is pre-recorded
# (same hash as commit-gate.sh), so only the docs check is pending.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p docs
cat > export.py <<'EOF'
import argparse

parser = argparse.ArgumentParser()
parser.add_argument("--old", action="store_true")
EOF
cat > docs/guide.md <<'EOF'
# Guide

Add `--old` to include archived records.
EOF
git add -A
git commit -qm "Add exporter"
sed -i 's/"--old"/"--new"/' export.py
git add -A
git diff --cached | sha256sum | cut -d' ' -f1 > .git/review-stamp
