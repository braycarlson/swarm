set shell := ["cmd", "/c"]

clear := if os() == "windows" { "cls" } else { "clear" }

default:
    @just --list

build:
    cargo build

release:
    {{clear}} && cargo build --release

run *args:
    cargo run -- {{args}}

check:
    cargo check

clippy:
    cargo clippy -- -D warnings

fmt:
    cargo fmt

fmt-check:
    cargo fmt -- --check

test:
    cargo test

clean:
    cargo clean
