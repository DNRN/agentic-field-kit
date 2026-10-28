#!/usr/bin/env bash
# Renders fox.png, the logo the installer shows, from fox.svg (copied unmodified
# from entropyfox.com, site/static/img/fox.svg). The navy tile is the Entropy
# Fox brand background (#144b74); it keeps the dark legs visible on dark
# terminals. Needs rsvg-convert (librsvg).
set -euo pipefail
cd "$(dirname "$0")"
rsvg-convert --width 480 --background-color '#144b74' fox.svg --output fox.png
