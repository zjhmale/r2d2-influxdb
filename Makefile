.PHONY: install example

install:
	cargo install --path . -f

example:
	cargo run --example ping
