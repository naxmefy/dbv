SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

default: build

pre-check:
	@rustc --version >/dev/null 2>&1 || (echo "ERROR: rust is required."; exit 1)
	@cargo --version >/dev/null 2>&1 || (echo "ERROR: cargo is required."; exit 1)
build: pre-check
	cargo build --release
test: pre-check
	cargo test
check: pre-check
	cargo check
	cargo fmt --check

book-check:
	@mdbook --version >/dev/null 2>&1 || (echo "ERROR: mdbook is required."; exit 1)
book-build: book-check
	mdbook build book
bb: book-build
book-serve: book-check
	mdbook serve --open book
bs: book-serve
