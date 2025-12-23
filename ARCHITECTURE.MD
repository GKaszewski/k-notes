# K-Note

## Executive Summary

A high-performance, self-hosted note-taking engine designed to replicate the Google Keep experience. Built with **Rust**, it prioritizes speed, memory safety, and long-term maintainability through **Hexagonal Architecture** (Ports and Adapters). The system is designed to be "storage-agnostic" and "format-blind," allowing users to own their data in simple formats like Markdown.

## Architecture Pattern: Hexagonal (Ports & Adapters)

We are decoupling the **Domain Logic** (the "What") from the **Infrastructure** (the "How").

- **The Core:** Contains `Note` and `User` entities and the business rules (e.g., "A note cannot have more than 10 tags").
- **Ports:** Traits that define how the core communicates with the outside world (e.g., `NoteRepository` or `AuthService`).
- **Adapters:** Concrete implementations (e.g., `SqliteNoteRepository`, `Argon2Auth`).
- **Why:** This allows us to start with REST/SQLite today and pivot to WebSockets/PostgreSQL tomorrow by simply writing a new adapter.

## Tech Stack

|     |     |     |
| --- | --- | --- |
| **Component** | **Primary Recommendation** | **Alternative** |
| **Language** | **Rust** (1.75+) | Go  |
| **API Framework** | **Axum** | Actix-Web |
| **Database** | **SQLite (via SQLx)** | PostgreSQL |
| **Search** | **SQLite FTS5** | Meilisearch |
| **Authentication** | **Axum Login** | OIDC (Keycloak/Authelia) |
| **Frontend** | **React + Tailwind + Shadcn/ui** | Vue + Radix |

## Data Model (Entity Relationship)

Code snippet

```
erDiagram
    USER ||--o{ NOTE : owns
    USER {
        uuid id PK
        string email
        string password_hash
    }
    NOTE ||--o{ TAG : contains
    NOTE {
        uuid id PK
        uuid user_id FK
        string title
        text content
        datetime created_at
        datetime updated_at
        boolean is_pinned
        boolean is_archived
    }
    TAG {
        uuid id PK
        string name
        uuid user_id FK
    }
```

## Folder Structure (Workspace Layout)

We will use a Cargo Workspace to enforce strict boundaries.

Plaintext

```
.
├── Cargo.toml
├── Makefile                 # Task runner for DB migrations, builds, and tests
├── crates
│   ├── domain               # Pure logic, Traits (Ports), and Entities (No SQLx here)
│   ├── infra                # Adapters: SQLx implementations, Email, Auth logic
│   └── api                  # Axum routes, Request/Response DTOs, Middleware
├── migrations               # SQLx migration files
└── docker-compose.yaml      # For easy self-hosting deployment
```

## API Design (MVP)

- `GET /api/v1/notes` - List notes (with filter for archived/pinned).
- `POST /api/v1/notes` - Create a new note (Accepts Markdown).
- `PATCH /api/v1/notes/:id` - Partial updates.
- `GET /api/v1/search?q=query` - Full-text search via FTS5.

## Guidelines & Principles

- **Error Handling:** Use `thiserror` for internal library errors and `anyhow` for high-level application flow. Map domain errors to specific HTTP status codes in the API layer.
- **Dependency Injection:** We will use **Atomic References (**`Arc<dyn Repository>`**)** to inject adapters into Axum state.
- **Validation:** All incoming data must be validated at the API boundary using `validator` crate before reaching the Domain.
- **DX (Developer Experience):** A `Makefile` is mandatory for one-command setups: `make setup`, `make dev`, `make test`.