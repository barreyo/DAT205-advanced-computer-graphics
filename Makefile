
.PHONY: build run clean

all: build run

build:
	@cargo build --release --features="winit glium"

run:
	@cargo run --release --features="winit glium"

clean:
	@cargo clean
	@find . -name "tmp*" -type f -delete
