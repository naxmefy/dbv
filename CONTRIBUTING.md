# Contributing to dbv

Thanks for your interest! dbv is in an early stage, so please open an issue or discussion before starting larger work.
By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

## Prerequisites

- Rust 1.88 or newer (the repository pins `stable` in `rust-toolchain.toml`)
- Optional, for the documentation book: [mdbook](https://github.com/rust-lang/mdBook) 0.5.4 and [mdbook-mermaid](https://github.com/badboy/mdbook-mermaid) 0.17.1

## Make targets

| Target | Description |
| --- | --- |
| `make build` | Release build |
| `make test` | Run the tests |
| `make check` | `cargo fmt --check` and `cargo clippy` with warnings as errors |
| `make book-build` (`make bb`) | Build the documentation book |
| `make book-serve` (`make bs`) | Serve the book locally |

## Workflow

1. Fork the repository and create a branch.
2. Make your change; run `make check` and `make test`.
3. Open a pull request against `main`.

### Pull request titles

The PR title must follow [Conventional Commits](https://www.conventionalcommits.org/), e.g. `feat(sqlite): add table browser`.
Allowed types: `feat`, `fix`, `docs`, `build`, `chore`, `refactor`, `test`, `perf`, `ci`.
CI checks the title.

PRs are squash-merged: the PR title and description become the commit message.
The changelog is generated automatically from these messages, so do not edit it by hand.

## Architecture decision records (ADRs)

Significant architectural decisions are recorded as ADRs in [`docs/adr/`](docs/adr/).
Write one when a change introduces or replaces a core dependency, changes a public interface or data format, or makes a trade-off that is hard to reverse.
Copy [`docs/adr/template.md`](docs/adr/template.md) and number the new file sequentially (`0001-...`, `0002-...`).
