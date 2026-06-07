# K-Notes

A self-hosted note-taking engine built in Rust with a strict hexagonal + DDD architecture.

![K-Notes Logo](k-notes-frontend/public/logo.png)

## Features

- **Authentication** — JWT-based login and registration (disable registration via `ALLOW_REGISTRATION=false`)
- **Note Management** — create, update, pin, archive, delete, version history
- **Markdown** — content stored and served as Markdown
- **Tagging** — user-scoped tags with get-or-create semantics
- **Search** — full-text search via SQLite FTS5
- **Smart Features** — semantic similarity links between notes using local embeddings (fastembed) and Qdrant; enabled when `QDRANT_URL` is set
- **Export / Import** — portable JSON backup and restore
- **API Docs** — Swagger UI at `/docs`, Scalar at `/scalar`
- **SPA** — React frontend served at `/` by the same process

## Tech Stack

### Backend
| Layer | Technology |
|-------|-----------|
| Language | Rust |
| HTTP | Axum 0.8 |
| Database | SQLite (sqlx + FTS5) |
| Events | NATS JetStream (prod) · in-memory bus (dev) |
| Embeddings | fastembed (AllMiniLML6V2) |
| Vector store | Qdrant |
| Auth | JWT (jsonwebtoken + argon2) |
| API docs | utoipa + Scalar + Swagger UI |

### Frontend
| Layer | Technology |
|-------|-----------|
| Framework | React + Vite |
| Language | TypeScript |
| Styling | Tailwind CSS + shadcn/ui |
| State | TanStack Query |
| Package manager | Bun |

## Architecture

The backend follows **Hexagonal Architecture + CQRS**:

```
crates/
  domain/               # Entities, value objects, ports (traits)
  application/          # Use cases (commands + queries), WorkerService
  adapters/
    sqlite/             # NoteRepository, TagRepository, UserRepository, LinkRepository
    auth/               # Argon2PasswordHasher, JwtValidator, OidcService
    nats/               # NatsEventPublisher, NatsEventConsumer (JetStream)
    event-publisher-memory/  # In-memory bus for dev/test
    event-payload/      # DomainEvent ↔ wire format (JSON)
    fastembed/          # EmbeddingGenerator implementation
    qdrant/             # VectorStore implementation
  wiring/               # Assembles AppContext from env vars
  presentation/         # Axum routes, OpenAPI, SPA serving
  api-types/            # Request/response DTOs (no domain dependency)
  bootstrap/            # HTTP server binary
  worker/               # Background event processor binary
```

Dependency direction: `domain ← application ← {presentation, worker}`. Adapters depend on domain only. `wiring` assembles everything.

## Getting Started

### Docker (recommended)

```bash
docker compose up -d
```

- **App + API**: http://localhost:3000
- **API docs**: http://localhost:3000/docs

### Local development

#### Prerequisites

- Rust stable (`rustup update stable`)
- Bun (`curl -fsSL https://bun.sh/install | bash`)

#### Quickstart

```bash
cp .env.example .env        # edit JWT_SECRET at minimum
make dev                    # API server on :3000
make dev-frontend           # Vite dev server on :5173 (separate terminal)
make dev-worker             # background worker (separate terminal, optional)
```

The API server also serves the SPA if you run `make build-frontend` first and `SPA_DIR` points at the dist directory. For active frontend development, use the Vite dev server instead — it hot-reloads and proxies API calls to `:3000`.

#### Environment variables

| Variable | Process | Required | Default | Description |
|----------|---------|----------|---------|-------------|
| `DATABASE_URL` | both | yes | — | SQLite path, e.g. `sqlite:data.db?mode=rwc` |
| `JWT_SECRET` | backend | yes | — | HS256 signing secret — generate with `openssl rand -hex 32` |
| `NATS_URL` | both | no | — | NATS JetStream URL; in-memory bus used if unset |
| `QDRANT_URL` | both | no | — | Enables smart features (semantic links) |
| `ENABLE_EMBEDDINGS` | **worker only** | no | `false` | Set `true` in the worker to load the fastembed model (~150 MB). Leave unset in the backend to save memory. |
| `QDRANT_COLLECTION` | both | no | `notes` | Qdrant collection name |
| `QDRANT_VECTOR_SIZE` | both | no | `384` | Must match the embedding model output dimension |
| `ALLOW_REGISTRATION` | backend | no | `true` | Set `false` to close public registration |
| `SPA_DIR` | backend | no | `k-notes-frontend/dist` | Path to built frontend; set empty for API-only mode |
| `CORS_ORIGINS` | backend | no | — | Comma-separated allowed origins |
| `PORT` | backend | no | `3000` | HTTP listen port |
| `HOST` | backend | no | `0.0.0.0` | HTTP listen address |
| `NATS_MAX_DELIVER` | worker | no | `5` | JetStream dead-letter threshold |
| `SMART_NEIGHBOUR_LIMIT` | worker | no | `10` | Max semantic links per note |
| `SMART_MIN_SIMILARITY` | worker | no | `0.7` | Cosine similarity threshold |

See `.env.example` for a commented template.

## API

All endpoints are under `/api/v1`. Full interactive docs at `/docs` (Swagger) or `/scalar` after starting the server.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/auth/login` | — | Login, returns JWT |
| POST | `/auth/register` | — | Register (if enabled) |
| GET | `/auth/me` | ✓ | Current user |
| GET | `/config` | — | Server capabilities |
| GET | `/notes` | ✓ | List notes (filter: pinned, archived, tag) |
| POST | `/notes` | ✓ | Create note |
| GET | `/notes/:id` | ✓ | Get note |
| PATCH | `/notes/:id` | ✓ | Update note |
| DELETE | `/notes/:id` | ✓ | Delete note |
| PATCH | `/notes/:id/pin` | ✓ | Pin / unpin |
| PATCH | `/notes/:id/archive` | ✓ | Archive / unarchive |
| GET | `/notes/:id/versions` | ✓ | Version history |
| GET | `/notes/:id/related` | ✓ | Semantically related notes |
| POST | `/notes/:id/tags` | ✓ | Add tag by name |
| DELETE | `/notes/:id/tags/:tag_id` | ✓ | Remove tag |
| GET | `/search?q=` | ✓ | Full-text search |
| GET | `/tags` | ✓ | List tags |
| POST | `/tags` | ✓ | Create tag |
| DELETE | `/tags/:id` | ✓ | Delete tag |
| PATCH | `/tags/:id` | ✓ | Rename tag |
| GET | `/export` | ✓ | Export all data as JSON |
| POST | `/import` | ✓ | Import from backup JSON |

## Deployment

```bash
# Build and push image
make deploy

# Or manually
IMAGE=your-registry/k-notes:latest make docker-build docker-push
```

The `CMD` in the Dockerfile starts `bootstrap` (HTTP server). Run `./worker` as a separate container or process for background event processing.

**Docker Compose** volumes to mount:
- `/app/data` — SQLite database file
- `/app/data/model-cache` — fastembed model cache (avoids re-download on restart)

## Project Structure

```
crates/                     # New architecture (active)
  adapters/                 # Infrastructure adapters
  api-types/                # HTTP DTOs
  application/              # Use cases + WorkerService
  bootstrap/                # API server binary
  domain/                   # Core domain
  presentation/             # Axum + OpenAPI + SPA
  wiring/                   # Dependency assembly
  worker/                   # Event worker binary
k-notes-frontend/           # React SPA
migrations/                 # Legacy migration files (see crates/adapters/sqlite/migrations/)

notes-api/                  ⚠ DEPRECATED — will be removed in a future release
notes-domain/               ⚠ DEPRECATED — will be removed in a future release
notes-infra/                ⚠ DEPRECATED — will be removed in a future release
notes-worker/               ⚠ DEPRECATED — will be removed in a future release
```

> **Note**: The `notes-*` directories contain the original monolithic implementation and are kept for reference only. They are excluded from the workspace and are not built. All active development happens in `crates/`.

## License

MIT — Copyright (c) 2025-2026 Gabriel Kaszewski
