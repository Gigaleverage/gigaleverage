run:
	@cargo run --bin gigaleverage

dev:
	@cargo watch -x run

# Clean the project
clean:
	@cargo clean 