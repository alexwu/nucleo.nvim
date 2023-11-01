bin_name := if os() == "macos" { "libnucleo_nvim" } else { "nucleo_nvim" }
bin_ext := if os() == "macos" { "dylib" } else { if os() == "windows" { "dll" } else { "so" } }
bin_ext_output := if os() == "windows" { "dll" } else { "so" }

default: release

lint:
    selene .

fmt:
    stylua .
    cargo clippy --fix
    cargo +nightly fmt --all

clean-lua:
    rm -f ./lua/nucleo_nvim.{{ bin_ext_output }}

clean-cargo:
    cargo clean

clean: clean-lua clean-cargo

build: clean-lua
    cargo build
    cp ./target/debug/{{ bin_name }}.{{ bin_ext }} ./lua/nucleo_nvim.{{ bin_ext_output }}

release: clean-lua
    cargo build --release
    cp ./target/release/{{ bin_name }}.{{ bin_ext }} ./lua/nucleo_nvim.{{ bin_ext_output }}

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
