#!/usr/bin/env bash
set -euo pipefail
# Staged diff renames local variables only: no observable change.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p docs
cat > stats.py <<'EOF'
# Arithmetic mean; 0.0 for an empty list.
def mean(values: list[float]) -> float:
    t = 0.0
    for v in values:
        t += v
    return t / len(values) if values else 0.0
EOF
cat > docs/stats.md <<'EOF'
# stats

`mean(values)` returns the arithmetic mean, or `0.0` for an empty list.
EOF
git add -A
git commit -qm "Add mean"
cat > stats.py <<'EOF'
# Arithmetic mean; 0.0 for an empty list.
def mean(values: list[float]) -> float:
    total = 0.0
    for value in values:
        total += value
    return total / len(values) if values else 0.0
EOF
git add -A
