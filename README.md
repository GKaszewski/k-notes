# K-Notes

A modern, self-hosted note-taking application built with performance, security, and clean architecture in mind.

![K-Notes Logo](k-notes-frontend/public/logo.png)

## Features

- **Authentication**: Secure user registration and login.
- **Note Management**: Create, edit, pin, archive, and delete notes.
- **Rich Text**: Markdown support for note content.
- **Version History**: Track changes, view history, note diffs, download versions, and restore previous states.
- **Organization**: Tagging system for easy filtering.
- **Theme**: Dark and Light mode support.
- **Responsive**: Mobile-friendly UI built with Tailwind CSS.
- **Architecture**:
    - **Backend**: Hexagonal Architecture (Domain, Infra, API layers) in Rust.
    - **Infrastructure**: Configurable database backends (SQLite, Postgres).
    - **Frontend**: Modern React with TypeScript and Vite.
- **Deployment**: Full Docker support with `compose.yml`.

## Tech Stack

### Backend
- **Language**: Rust
- **Framework**: Axum
- **Database**: SQLite (Default) or Postgres (Supported via feature flag)
- **Dependency Injection**: Manual wiring for clear boundaries

### Frontend
- **Framework**: React + Vite
- **Language**: TypeScript
- **Styling**: Tailwind CSS + Shadcn UI
- **State Management**: TanStack Query (React Query)

## Getting Started

### Docker (Recommended)

Run the entire stack with a single command:

```bash
docker compose up -d --build
```

- **Frontend**: http://localhost:8080
- **Backend**: http://localhost:3000

The frontend is automatically configured to talk to the backend.

### Local Development

#### Backend

1.  Navigate to the `notes-api` directory (or root).
2.  Set up the environment variables (see `.env.example`).
3.  Run the server:

```bash
cargo run -p notes-api
```


By default, this uses the **SQLite** backend.

#### Configuration

The application is configured via environment variables (or `.env` file):

-   `ALLOW_REGISTRATION`: Set to `false` to disable new user registration (default: `true`).
-   `DATABASE_URL`: Connection string for the database.
-   `SESSION_SECRET`: Secret key for session encryption.
-   `CORS_ALLOWED_ORIGINS`: Comma-separated list of allowed origins.

**Running with Postgres:**

To use PostgreSQL, build with the `postgres` feature:
```bash
cargo run -p notes-api --no-default-features --features notes-infra/postgres
```
*Note: Ensure your `DATABASE_URL` is set to a valid Postgres connection string.*

#### Frontend

1.  Navigate to `k-notes-frontend`.
2.  Install dependencies:

```bash
bun install
```

3.  Run the dev server:

```bash
bun dev
```

## Database Architecture

The backend follows a Hexagonal Architecture (Ports and Adapters). The `notes-domain` crate defines the repository capabilities (Ports), and `notes-infra` implements them (Adapters).

### Supported Databases
- **SQLite**: Fully implemented (default). Ideal for single-instance, self-hosted deployments.
- **Postgres**: Structure is in place (via feature flag), ready for implementation.

### Extending Database Support

To add a new database (e.g., MySQL), follow these steps:

1.  **Dependencies**: Add the driver to `notes-infra/Cargo.toml` (e.g., `sqlx` with `mysql` feature) and create a feature flag.
2.  **Configuration**: Update `DatabaseConfig` in `notes-infra/src/db.rs` to handle the new connection URL scheme and connection logic in `create_pool`.
3.  **Repository Implementation**:
    - Implement `NoteRepository`, `TagRepository`, and `UserRepository` traits for the new database in `notes-infra`.
4.  **Factory Integration**:
    - Update `notes-infra/src/factory.rs` to include a builder for the new repositories.
    - Update `build_database_pool` and repository `build_*` functions to support the new database type match arm.
5.  **Migrations**:
    - Add migration files in `migrations/<db_type>`.
    - Update `run_migrations` in `db.rs` to execute them.

This design ensures the `notes-api` layer remains completely agnostic to the underlying database technology.

## Project Structure

```
├── notes-api       # API Interface (Axum, HTTP routes)
├── notes-domain    # Core Business Logic (Entities, Services, Ports)
├── notes-infra     # Infrastructure (Database adapters, Repositories)
├── k-notes-frontend # React Frontend Application
├── migrations      # SQLx Database Migrations
└── compose.yml     # Docker Composition
```

## License

MIT
