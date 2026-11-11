# 0022. afk checks for new releases and `--update` updates the binary too

Date: 2026-11-11

## Status

Accepted

Supersedes the updating part of [ADR 0010](0010-components-embedded-in-binary.md) ("Updating is: upgrade the binary, then `--update` in each project"). Plan: [afk updates itself](../plans/self-update.md).

## Context

ADR 0010 builds the components into the binary, so a component change reaches users only through a new binary. Updating takes two steps: rerun the install script, then run `afk --update` in each project. Nothing tells users that a release exists, so they don't upgrade. On 2026-11-11 the maintainer's own machine still ran 0.4.0, a day after 0.5.0 was released.

dist's shell and PowerShell installers already write an install receipt (`~/.config/agentic-field-kit/agentic-field-kit-receipt.json`, or `%LOCALAPPDATA%` on Windows) with the install prefix, the version, and the GitHub source. `install-updater = false` only leaves out dist's separate updater binary. dist's [axoupdater](https://github.com/axodotdev/axoupdater) library reads that receipt, finds the latest GitHub release (the `installer/vX.Y.Z` tags parse), and runs that release's installer into the same prefix. It refuses when the running binary isn't the one the receipt describes. `cargo install` and `--source` builds have no receipt.

ADR 0010 rejected downloading *components* at run time, because that meant network access on every run and a second release channel. A version check is also network access at run time, but only to compare versions: components still come from the binary, and the release channel stays the same.

## Decision

We will make afk update itself through dist's receipt and the `axoupdater` crate:

- **Check.** The menu and the `--list` header show "X.Y.Z available · `afk --update`" when a newer release exists. The check runs only on a terminal and never delays the menu: it runs in the background with a short timeout, and the result is cached for 24 hours. Errors are silent; offline, the last version a check found is shown. It is skipped with `--source` or `AFK_NO_UPDATE_CHECK=1`.
- **`afk --update`** first updates the binary when a newer release exists and afk was installed by a dist installer. It then re-runs the *new* binary with the same arguments, so components are refreshed from the new release. Outside a project it updates the binary only. `--dry-run` shows the binary step without downloading, and `--yes` skips its confirmation.
- **Installs without a receipt** (`cargo install`, a local build): `--update` prints the command that updates them and refreshes components as before. afk never replaces a binary it didn't install. With `--source` (contributors) there is no binary step.

## Consequences

- One command updates everything, and users see new releases without visiting GitHub.
- The first network access in the binary. Offline use, proxies, and CI must stay unaffected. The check is cached and silent, and tests must not reach the network.
- The binary grows from 1.4 MB to about 6.7 MB. In a spike, axoupdater 0.10.3 with `blocking` added 5.3 MB to a stripped hello-world, through reqwest, tokio, image, and rustls with aws-lc-rs. aws-lc-sys is C code that every dist target, including aarch64 musl, must build. dist's PR builds (`pr-run-mode = "upload"`) show a failure before a release.
- The self-update runs the release's installer script. The shell installer checks each archive's sha256 against values in the script; the 0.5.0 PowerShell installer does no checksum check. Integrity therefore rests on HTTPS to GitHub, as it does for a first install today.
- Users on 0.5.0 or older must reinstall once by hand. The feature works from the release after it ships.

## Rejected alternatives

- **Keep ADR 0010's two-step update**: users keep missing releases. This is the problem the plan exists to solve.
- **ureq plus our own update code**: a spike with ureq 3 and self-replace added 2.0 MB (rustls with ring, no tokio). afk would fetch `releases/latest` (a redirect to the tag, no API rate limit), download that release's installer, and run it into the receipt's prefix. That repeats about 100 lines of axoupdater and its edge cases (old receipt layouts, Windows paths, checking that the receipt belongs to this executable), which we would then maintain ourselves.
- **Shell out to `curl | sh` or PowerShell**: no new crates, but it depends on tools on `PATH` and has weak error handling.
- **dist's separate updater binary (`install-updater = true`)**: a second executable on `PATH`, and still no notice in the UI.
- **A separate `--self-update` flag**: `--update` stays as it is, but the user asked for one command.
- **Run `cargo install` for cargo installs**: slow, needs a toolchain, and replaces a build the user chose to make.
