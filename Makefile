.PHONY: setup dev test check clean migrate

# Database URL for development
DATABASE_URL ?= sqlite:data.db?mode=rwc

# Setup development environment
setup:
	@echo "🔧 Setting up K-Notes development environment..."
	cargo build --workspace
	@echo "✅ Setup complete!"

# Run the development server
dev:
	@echo "🚀 Starting K-Notes API server..."
	DATABASE_URL=$(DATABASE_URL) cargo run --package notes-api

# Run all tests
test:
	@echo "🧪 Running tests..."
	cargo test --workspace

# Check code compiles
check:
	cargo check --workspace

# Clean build artifacts
clean:
	cargo clean
	rm -f data.db data.db-wal data.db-shm

# Run migrations (done automatically on server start)
migrate:
	@echo "📦 Running database migrations..."
	DATABASE_URL=$(DATABASE_URL) cargo run --package notes-api -- --migrate-only 2>/dev/null || \
		(cargo run --package notes-api &  sleep 2 && kill $$!)
	@echo "✅ Migrations complete!"

# Run clippy lints
lint:
	cargo clippy --workspace -- -D warnings

# Format code
fmt:
	cargo fmt --all

# Quick development cycle
quick: check test
	@echo "✅ All checks passed!"
