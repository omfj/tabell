build:
    cargo build --release

install: build
    cp target/release/tabell ~/.local/bin/tabell