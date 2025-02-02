.PHONY: all
all: clean build clippy test

.PHONY: build
build:
	@echo "\n### $@"
	cargo build --release

.PHONY: clean
clean:
	@echo "\n### $@"
	cargo clean

.PHONY: clippy
clippy:
	@echo "\n### $@"
	cargo clippy

.PHONY: test
test:
	@echo "\n### $@"
	cargo test --workspace

.PHONY: format
format:
	@echo "\n### $@"
	cargo fmt