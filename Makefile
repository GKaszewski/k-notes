.DEFAULT_GOAL := check

DATABASE_URL ?= sqlite:data.db?mode=rwc
IMAGE        ?= registry.gabrielkaszewski.dev/k-notes:latest

# ── Local dev ─────────────────────────────────────────────────────────────────

# Start the API server. JWT_SECRET is required; this default is dev-only.
dev:
	DATABASE_URL=$(DATABASE_URL) JWT_SECRET=dev-secret-not-for-production-use \
	  cargo run -p bootstrap

# Start the background worker (uses in-memory bus when NATS_URL is unset).
dev-worker:
	DATABASE_URL=$(DATABASE_URL) cargo run -p worker

# Start the Vite dev server for the frontend (expects the API on :3000).
dev-frontend:
	cd k-notes-frontend && bun run dev

# Build the frontend SPA into k-notes-frontend/dist.
build-frontend:
	cd k-notes-frontend && bun install && bun run build

# ── Quality checks ────────────────────────────────────────────────────────────

# Run the full check suite (same order as CI).
check: fmt-check clippy test
	@echo "✅ All checks passed"

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

clippy:
	cargo clippy --workspace -- -D warnings

test:
	cargo test --workspace

# Apply fmt + clippy fixes in one shot.
fix:
	cargo fmt
	cargo clippy --fix --allow-dirty --allow-staged

# ── Docker ────────────────────────────────────────────────────────────────────

docker-build:
	docker buildx build --platform linux/amd64 -t $(IMAGE) .

docker-push:
	docker push $(IMAGE)

deploy:
	IMAGE=$(IMAGE) ./deploy.sh

# ── Housekeeping ──────────────────────────────────────────────────────────────

clean:
	cargo clean
	rm -f data.db data.db-wal data.db-shm

.PHONY: dev dev-worker dev-frontend build-frontend check fmt fmt-check clippy test fix \
        docker-build docker-push deploy clean
