# 0009. The installer is a Rust binary with a terminal UI

Date: 2026-10-28

## Status

Proposed

Plan: [Rust installer](../plans/rust-installer.md). Related: [ADR 0006](0006-project-scope-install-default.md) (install scope), [ADR 0007](0007-component-catalog.md) (catalog), [ADR 0008](0008-project-install-manifest.md) (manifest), [ADR 0010](0010-components-embedded-in-binary.md) (what the binary carries).

## Context

`install.sh` (0.2.0) installs, updates, and removes components per project, with a numbered `read` menu. Three limits remain:

- **The menu is basic.** Users want to see what each component does, what it needs, and what a change will do, with a real interface and the Entropy Fox brand.
- **Windows needs Git Bash or WSL** ([ADR 0006](0006-project-scope-install-default.md), consequences).
- **A UI on top of bash means two implementations.** Every safety rule (never touch foreign files, never overwrite an edit without `--force`, ask before removing what the user didn't name) lives in `install.sh`. A separate UI that writes files would have to repeat them.

Options considered: an OpenTUI front end (needs Bun 1.3.14+ or Node 26.4+ with experimental FFI; 0.6.x), a Rust front end over `install.sh`, and a Rust binary that replaces `install.sh`. `tests/install.test.sh` tests the installer only through its command line, so it can check a second implementation unchanged.

## Decision

We will replace `install.sh` with one Rust binary, `agentic-toolkit`, that is both the command line and the terminal UI:

- **Same contract.** The command-line flags, the `--list` table, the manifest format, and the content hash of [ADR 0008](0008-project-install-manifest.md) stay byte-compatible, so manifests written by `install.sh` 0.2.x keep working.
- **One engine.** Status, planning, and applying live in a library module; the command line and the UI only build requests and show plans. The UI never writes files itself.
- **UI:** [Ratatui](https://ratatui.rs) with the crossterm backend (Linux, macOS, Windows). It opens when the binary runs on a terminal without arguments.
- **Acceptance:** `tests/install.test.sh` runs against the binary through a two-line wrapper; the port is done when every check passes on Linux, macOS, and Windows. UI behaviour is tested in Rust with Ratatui's `TestBackend`.
- `install.sh` stays until then, and is removed in the same release that ships the binary.

## Consequences

- One implementation of every rule; Windows runs natively, without Git Bash.
- Contributors need a Rust toolchain to change the installer; component authors don't.
- Releases build binaries for each platform ([ADR 0010](0010-components-embedded-in-binary.md)).
- ADR 0007's reason for TSV ("so the installer can stay in bash") no longer applies; the catalog stays TSV because it is readable, diffable, and already tested.
- ADR 0006's consequence "the installer is bash; on Windows it runs in Git Bash or WSL" is superseded once this is accepted.

## Rejected alternatives

- **OpenTUI front end over `install.sh`**: a JavaScript runtime on every user's machine (Bun or Node 26.4+ with `--experimental-ffi`), a pre-1.0 API, and still Git Bash on Windows.
- **Rust front end over `install.sh`**: two languages, the binary still needs bash and a clone, and Windows still needs Git Bash.
- **Keep the bash menu and polish it**: no layout, no scrolling, no image support for the logo; the Windows limit stays.
- **Cursive or iocraft instead of Ratatui**: Cursive suits dialog forms but not a custom branded screen; iocraft is younger with a much smaller ecosystem. Ratatui has the largest ecosystem, including `ratatui-image` for the logo.
