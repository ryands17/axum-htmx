dev:
	cargo watch -c -w app/src -x 'run -p app'
	
build:
	cargo build --release

start:
	./target/release/app