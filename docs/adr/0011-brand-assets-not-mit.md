# 0011. Entropy Fox brand assets are not covered by the MIT licence

Date: 2026-10-28

## Status

Proposed

Plan: [Rust installer](../plans/rust-installer.md). Related: [ADR 0009](0009-installer-in-rust.md).

## Context

The installer shows the Entropy Fox logo and name, so the logo file has to live in this repository and inside the binary. The repository is MIT-licensed: anything committed without an exception may be reused, modified, and redistributed by anyone, including the logo. A licence that has been granted for a released file can't be taken back later.

## Decision

We will keep the Entropy Fox name and logo in `assets/brand/`, with their own `LICENSE`: all rights reserved, except that they may be shown unmodified as part of this toolkit and its forks. The root `LICENSE` and the README state that MIT covers everything except `assets/brand/`. The logo is derived from `entropyfox.com`'s `site/static/img/fox.svg` and never redrawn here.

## Consequences

- The toolkit stays MIT; the logo and name stay the owner's.
- A fork that wants its own identity replaces `assets/brand/` and the name constant; the installer must not depend on the brand for anything but display.
- Contributors can't change the logo through a pull request; changes come from `entropyfox.com`.

## Rejected alternatives

- **Logo under MIT with the rest**: anyone could reuse or alter the logo for anything, permanently.
- **Creative Commons (for example CC BY-ND)**: still allows reuse outside this toolkit; more than display needs.
- **Download the logo at run time from entropyfox.com**: network on every start, and the binary breaks offline or when the site changes.
