set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

install:
    cargo install --path backend --force
    cargo install --path cli --force

build:
    cargo build --workspace

test:
    cargo test --workspace

check:
    cargo fmt --check --all
    cargo clippy --workspace
