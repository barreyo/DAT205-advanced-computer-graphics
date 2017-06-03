
.PHONY: build run clean

all: build run

build:
	@cargo build --release

run:
	@cargo run --release

clean:
	@cargo clean
	@find . -name "tmp*" -type f -delete
