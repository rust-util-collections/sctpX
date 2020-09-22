all: lint

lint:
	cargo clippy

build:
	cargo build

release:
	cargo build --release

fmt:
	bash ./tools/fmt.sh
