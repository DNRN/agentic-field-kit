# 0012. The installer draws the Entropy Fox as coloured ASCII

Date: 2026-10-28

## Status

Accepted

Plan: [Rust installer](../plans/rust-installer.md), R4. Supersedes, in [ADR 0011](0011-brand-assets-not-mit.md), "never redrawn here"; the rest of 0011 stands.

## Context

The installer shows `assets/brand/fox.png` through `ratatui-image`: Kitty, Sixel, or iTerm2 graphics in five terminals known to answer its query, half-blocks on other truecolor terminals, nothing anywhere else. No fox in macOS Terminal, tmux, the Linux console, 256-colour terminals, or under `NO_COLOR`. The query is limited to an allow-list because `ratatui-image` leaves a thread reading stdin until the terminal answers.

The owner wants the fox in every terminal. Converting `fox.svg` to ASCII by script was tried at 36 to 48 columns, matching character shapes against the silhouette, with brightness ramps and with facet edges: the low-poly facets turn into noise and the fox can't be recognised at header size.

## Decision

We will draw the logo as text: `assets/brand/fox.txt`, 41×12 printable ASCII traced by hand from `fox.svg`, and `assets/brand/fox.colors`, the same grid with one letter per character naming a brand shade (cream, light orange, orange, rust, brown). The installer draws the shades as xterm 256-colour indices, and the characters alone under `NO_COLOR`. The drawing is the logo's text form, by permission of the owner, and falls under `assets/brand/LICENSE` like the SVG; changes to it come from the owner, as for the SVG. `fox.png`, `render.sh`, `ratatui-image`, and `image` are removed.

## Consequences

- The fox shows in any terminal with a monospace font at 80×24 or more, coloured where the terminal has 256 colours (all current terminals, including Windows Terminal and conhost), in plain text under `NO_COLOR` or in a 16-colour console.
- No terminal query, no stdin thread, no allow-list; two fewer dependencies.
- Lower fidelity than the image: a silhouette in five shades, not the facets.
- The header grows from 8 to 12 rows; at 80×24 the component table keeps 10 rows, 7 of them for components.
- There are now two forms of the logo to keep in step when `entropyfox.com` changes it.
- A fork with its own logo also sets `LOGO_WIDTH` and `LOGO_HEIGHT` in `installer/src/brand.rs` to its size; the brand test fails until they match.

## Rejected alternatives

- **Keep the image, ASCII as the fallback**: two ways to draw, and the stdin thread and allow-list stay, for a better fox in five terminals.
- **Generate the ASCII from `fox.svg` with a script**: tried; not recognisable at header size (see Context).
- **Half-blocks in 256 colours, pre-rendered**: close to the image and works without truecolor, but it is Unicode block art, not ASCII, and the owner asked for ASCII.
