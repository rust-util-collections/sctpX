all: lint

lint:
	cargo clippy

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

fmt:
	bash ./tools/fmt.sh
