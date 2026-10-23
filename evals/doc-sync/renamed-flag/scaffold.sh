#!/usr/bin/env bash
set -euo pipefail
# docs/guide.md documents --old; the staged diff renames it to --new in code only.
git init -q -b main
git config user.name "Eval User"
git config user.email eval@example.com
mkdir -p docs
cat > export.py <<'EOF'
import argparse


def main() -> None:
    parser = argparse.ArgumentParser(description="Export records to CSV.")
    parser.add_argument("--old", action="store_true", help="include archived records")
    parser.add_argument("--out", default="export.csv", help="output file")
    args = parser.parse_args()
    print(args)


if __name__ == "__main__":
    main()
EOF
cat > README.md <<'EOF'
# exporter

Exports records to CSV. See [the guide](docs/guide.md).
EOF
cat > docs/guide.md <<'EOF'
# Export guide

## Usage

Run `python export.py --out records.csv` to export active records.

Add `--old` to include archived records:

    python export.py --old --out all.csv

## Troubleshooting

If the export hangs, check that the output directory is writable and not on a network share.
EOF
git add -A
git commit -qm "Add exporter"
sed -i 's/"--old"/"--new"/' export.py
git add -A
