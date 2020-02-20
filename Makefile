.PHONY: install example doc test

doc:
	cargo doc --no-deps

test:
	cargo test
	$(MAKE) example

install:
	cargo install --path . -f

example:
	cargo run --example ping
	cargo run --example read_write

publish:
	cargo publish
