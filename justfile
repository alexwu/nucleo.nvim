bin_name := if os() == "windows" { "nucleo_nvim" } else { "libnucleo_nvim" }
bin_ext := if os() == "macos" { "dylib" } else { if os() == "windows" { "dll" } else { "so" } }
bin_ext_output := if os() == "windows" { "dll" } else { "so" }

set dotenv-load := true

default: release

lint:
    selene .

fmt-lua:
    stylua .

fmt-rust:
    cargo +nightly fmt --all

fmt: fmt-lua fmt-rust

clean-cargo:
    cargo clean

clean-lua:
    rm -f ./lua/nucleo_nvim.{{ bin_ext_output }}
    rm -f ./lua/nucleo_rs.{{ bin_ext_output }}

clean: clean-lua clean-cargo

copy-debug-build-artifacts:
    cp ./target/debug/{{ bin_name }}.{{ bin_ext }} ./lua/nucleo_rs.{{ bin_ext_output }}

build: && clean-lua copy-debug-build-artifacts
    cargo +beta build

copy-release-build-artifacts:
    cp ./target/release/{{ bin_name }}.{{ bin_ext }} ./lua/nucleo_rs.{{ bin_ext_output }}

release: && clean-lua copy-release-build-artifacts
    cargo build --release

clippy:
    cargo clippy --all --all-targets --all-features

clippy-fix:
    cargo clippy --fix --allow-staged

fix:
    cargo fix --allow-dirty

clippy-pedantic:
    cargo clippy -- -W clippy::pedantic

check: clippy

pattern := ''

test PATTERN=pattern:
    RUST_LOG=trace cargo test {{ PATTERN }} --no-fail-fast

bench:
    cargo bench
