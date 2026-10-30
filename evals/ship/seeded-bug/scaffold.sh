#!/usr/bin/env bash
set -euo pipefail
# The staged change rewrites last_n with a loop that runs one past the end.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
printf 'def last_n(items, n):\n    """The last n items."""\n    return items[-n:] if n else []\n' > series.py
git add -A
git commit -qm "Add last_n"
cat > series.py <<'PY'
def last_n(items, n):
    """The last n items, without slicing."""
    out = []
    for i in range(len(items) - n, len(items) + 1):
        out.append(items[i])
    return out
PY
git add series.py
