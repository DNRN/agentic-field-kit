# 0010. Components ship inside the binary; releases are built with dist

Date: 2026-10-28

## Status

Accepted

Plan: [Rust installer](../plans/rust-installer.md). Related: [ADR 0008](0008-project-install-manifest.md) (manifest and hash), [ADR 0009](0009-installer-in-rust.md) (Rust installer).

## Context

With `install.sh`, a user clones the repository and the installer copies from the clone; `git pull` plus `--update` is the whole update. A compiled installer ([ADR 0009](0009-installer-in-rust.md)) has to get the components from somewhere:

- **From a clone next to the binary**: users need git, the clone, and a binary whose version matches the clone.
- **Downloaded at run time**: needs network access on every run, and a second versioning scheme.
- **Built into the binary**: one download, and the binary's version is the components' version.

Contributors and the black-box tests need to run the installer against a working tree they are editing, and `tests/install.test.sh` changes components "upstream" between runs.

Users without Rust need prebuilt binaries for Linux, macOS, and Windows. [dist](https://axodotdev.github.io/cargo-dist) (cargo-dist) generates a GitHub release workflow, per-platform archives with checksums, and `curl | sh` and PowerShell installers.

## Decision

We will build `catalog.tsv` and every component path it lists into the binary at compile time, and release with dist:

- The binary installs its built-in components by default. `--source DIR` reads a toolkit working tree instead; contributors and tests use it.
- The binary's version is the toolkit release: `Cargo.toml`, `.claude-plugin/plugin.json`, and `CHANGELOG.md` agree, and `tests/release.test.sh` checks it.
- Updating is: upgrade the binary, then `agentic-toolkit --update` in each project. The content hash of [ADR 0008](0008-project-install-manifest.md) still decides what is `outdated` or `modified`.
- Releases: dist builds Linux (x86_64, aarch64; musl, so no glibc dependency), macOS (x86_64, aarch64), and Windows (x86_64) on tag, with shell and PowerShell installers. `cargo install --git https://github.com/DNRN/agentic-toolkit` stays available for people with Rust.

## Consequences

- Getting started is one command; no git, no clone, no Bash.
- A component change reaches users only through a release; `--source` covers everything before that.
- The release pipeline is new work to maintain, and the binaries are unsigned; macOS warns only when a binary was downloaded through a browser.
- The installer no longer needs to tell the toolkit clone apart from a project in the common case; with `--source DIR`, it still refuses DIR as the project.

## Rejected alternatives

- **Read components from a clone (as `install.sh` does)**: keeps git and a clone as prerequisites, and a binary and clone of different versions would install a mix.
- **Download components at run time**: network on every run, offline installs fail, and a second release channel to secure.
- **Hand-written release workflow**: repeats what dist generates and keeps up to date (targets, archives, checksums, installers).
