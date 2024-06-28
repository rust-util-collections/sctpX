all: fmt lint

lint:
	cargo clippy

build:
	cargo build

release:
	cargo build --release

test:
	cargo test -- --nocapture --test-threads=1

fmt:
	@ bash ./tools/fmt.sh
