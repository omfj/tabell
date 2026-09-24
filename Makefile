.PHONY: build install

build:
	@cargo build --release

install:
	@cargo install --path .

check:
	@cargo clippy --all-targets --all-features -- -D warnings
	@cargo fmt --all -- --check
