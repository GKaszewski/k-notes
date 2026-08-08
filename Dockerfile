# ----- spa -----
FROM oven/bun:1-slim AS spa-builder
WORKDIR /spa
COPY k-notes-frontend/package.json k-notes-frontend/bun.lock ./
RUN bun install --frozen-lockfile
COPY k-notes-frontend/ .
RUN bun run build

# ----- build -----
FROM rust:slim-bookworm AS builder

WORKDIR /build

# NOTE: before building, remove the legacy crates (notes-api, notes-domain,
# notes-infra, notes-worker) from the [workspace] members in Cargo.toml —
# they depend on k-core (a private git repo) not accessible in CI/CD.

# Copy workspace manifests first so Docker can cache the dep-fetch layer.
COPY Cargo.toml Cargo.lock ./

COPY crates/adapters/auth/Cargo.toml                   crates/adapters/auth/Cargo.toml
COPY crates/adapters/event-payload/Cargo.toml          crates/adapters/event-payload/Cargo.toml
COPY crates/adapters/event-publisher-memory/Cargo.toml crates/adapters/event-publisher-memory/Cargo.toml
COPY crates/adapters/fastembed/Cargo.toml              crates/adapters/fastembed/Cargo.toml
COPY crates/adapters/nats/Cargo.toml                   crates/adapters/nats/Cargo.toml
COPY crates/adapters/qdrant/Cargo.toml                 crates/adapters/qdrant/Cargo.toml
COPY crates/adapters/sqlite/Cargo.toml                 crates/adapters/sqlite/Cargo.toml
COPY crates/api-types/Cargo.toml                       crates/api-types/Cargo.toml
COPY crates/application/Cargo.toml                     crates/application/Cargo.toml
COPY crates/bootstrap/Cargo.toml                       crates/bootstrap/Cargo.toml
COPY crates/domain/Cargo.toml                          crates/domain/Cargo.toml
COPY crates/presentation/Cargo.toml                    crates/presentation/Cargo.toml
COPY crates/wiring/Cargo.toml                          crates/wiring/Cargo.toml
COPY crates/worker/Cargo.toml                          crates/worker/Cargo.toml

# Stub every crate so Cargo can resolve and fetch all dependencies.
RUN find crates -name "Cargo.toml" | sed 's|/Cargo.toml||' | \
    xargs -I{} sh -c 'mkdir -p {}/src && printf "fn main(){}\n" > {}/src/main.rs && touch {}/src/lib.rs'

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libstdc++-12-dev \
 && rm -rf /var/lib/apt/lists/*

RUN cargo fetch

# Copy real source and adapter migrations.
COPY crates/ crates/
COPY crates/adapters/sqlite/migrations/ crates/adapters/sqlite/migrations/

RUN cargo build --release -p bootstrap

# ----- runtime -----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /build/target/release/bootstrap ./bootstrap

# Frontend dist — served at / by bootstrap when SPA_DIR is set.
COPY --from=spa-builder /spa/dist ./frontend/dist

EXPOSE 3000

ENV RUST_LOG=bootstrap=info,tower_http=info
ENV SPA_DIR=/app/frontend/dist

CMD ["./bootstrap"]
