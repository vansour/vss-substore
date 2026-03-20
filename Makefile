.PHONY: check clippy clippy-wasm serve build

check:
	cargo fmt --all -- --check
	cargo check --workspace
	cargo check -p submora-web --target wasm32-unknown-unknown

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

clippy-wasm:
	cargo clippy -p submora-web --target wasm32-unknown-unknown -- -D warnings

serve:
	dx serve --platform web --package submora-web

build:
	dx build --platform web --package submora-web --release
