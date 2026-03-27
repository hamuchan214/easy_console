.PHONY: install uninstall build release

install:
	cargo install --path .

uninstall:
	cargo uninstall easy_console

build:
	cargo build

release:
	cargo build --release
