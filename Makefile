.PHONY: all build test lint check-fmt check-clippy

all: test build

build:
	cargo build --release -p agent 

test:
	cargo test --release

clean:
	cargo clean

lint: \
	license \
	check-fmt \
	check-clippy

check-fmt:
	cargo fmt --all --check

check-clippy:
	cargo clippy --all -- -D warnings
