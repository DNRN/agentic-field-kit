---
type: regex
target: { source: file, path: shop/cart.py }
pattern: 'range\(1,'
match: not_contains
---

The off-by-one loop start is fixed.
