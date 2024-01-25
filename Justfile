# Default 'just' command
default:
    @echo "Available commands:"
    @echo "  build   - Compile the project"
    @echo "  run     - Run the project"
    @echo "  watch   - Watch for changes and rebuild"
    @echo "  doc     - Generate project documentation"
    @echo "  serve   - Serve documentation on a local server"

# Build the project
build:
    cargo build

# Run the project
run:
    cargo run

# Watch for changes and rebuild
watch:
    cargo watch -x run

# Generate project documentation
doc:
    cargo doc --no-deps

# Serve documentation on a local server
serve:
    cargo doc --document-private-items --no-deps --open
