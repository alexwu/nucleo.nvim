lint:
    selene .

fmt:
    stylua .
    cargo clippy --fix
    cargo +nightly fmt --all

clean:
    cargo clean

build:
    cargo build

build-release:
    cargo build --release

clippy:
    cargo clippy --all --all-targets --all-features

fix:
    cargo fix --allow-dirty

pedantic:
    cargo clippy -- -W clippy::pedantic

check: clippy

pattern := ''

test PATTERN=pattern:
    RUST_LOG=trace cargo test {{ PATTERN }} --no-fail-fast

