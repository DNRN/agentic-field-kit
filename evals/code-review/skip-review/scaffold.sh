#!/usr/bin/env bash
set -euo pipefail
# Staged behavior change; the user waives review in the prompt.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
cat > pricing.py <<'EOF'
def apply_discount(price: float, code: str) -> float:
    if code == "SAVE10":
        return round(price * 0.9, 2)
    return price
EOF
git add -A
git commit -qm "Add pricing"
cat > pricing.py <<'EOF'
def apply_discount(price: float, code: str) -> float:
    if code == "SAVE10":
        return round(price * 0.9, 2)
    if code == "VIP":
        return round(price * 0.7, 2)
    return price
EOF
git add -A
