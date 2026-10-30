#!/usr/bin/env bash
set -euo pipefail
# One staged typo fix in a string; no docs, so the gate's docs check is off.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
printf 'def greeting(name):\n    return f"Helo, {name}!"\n' > greet.py
git add -A
git commit -qm "Add greeting"
printf 'def greeting(name):\n    return f"Hello, {name}!"\n' > greet.py
git add greet.py
