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
    - **Frontend**: Modern React with TypeScript and Vite.
- **Deployment**: Full Docker support with `compose.yml`.

## Tech Stack

### Backend
- **Language**: Rust
- **Framework**: Axum
- **Database**: SQLite (SQLx)
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

## 🏗️ Project Structure

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
