# 0007. Releasing and merge strategy

Date: 2026-10-23

Status: Accepted

## Context

We want releases to be cheap, repeatable and not dependent on one person's laptop, with a changelog that writes itself and binaries for the common platforms. A linear, readable history on `main` helps both.

## Decision

- **Merging:** squash-only. The squash commit message is the pull request title plus description.
- **Conventional Commits:** the PR title must follow the Conventional Commits format, enforced by a CI check. Because every PR becomes one commit, the titles drive versioning and the changelog.
- **release-plz:** on every merge to `main` it maintains a release PR that bumps the SemVer version in `Cargo.toml` and `Cargo.lock` and updates `CHANGELOG.md` using git-cliff. Merging the release PR creates the tag `v{version}`.
- **cargo-dist:** a pushed tag builds binaries and creates a GitHub Release with them, using the matching `CHANGELOG.md` section as release notes. Targets are Linux (x86_64, aarch64), macOS (x86_64, aarch64) and Windows (x86_64), with shell and PowerShell installers.
- **crates.io:** publishing is deferred until the crates are ready to be public API.

## Consequences

- Releasing means reviewing and merging one PR; no manual tagging or changelog editing.
- Contributors need to write good PR titles; commits inside a PR can be messy.
- Squash-only must be enforced in the repository settings, and `main` protected, by the maintainer.
- Distribution through package managers is a later milestone and builds on these artifacts.
