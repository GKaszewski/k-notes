//! K-Notes Infrastructure Layer
//!
//! This crate provides concrete implementations (adapters) for the
//! repository ports defined in the domain layer.
//!
//! ## Adapters
//!
//! - [`SqliteNoteRepository`] - SQLite adapter for notes with FTS5 search
//! - [`SqliteUserRepository`] - SQLite adapter for users (OIDC-ready)
//! - [`SqliteTagRepository`] - SQLite adapter for tags
//!
//! ## Database
//!
//! - [`db::create_pool`] - Create a database connection pool
//! - [`db::run_migrations`] - Run database migrations

pub mod db;
pub mod note_repository;
pub mod tag_repository;
pub mod user_repository;

// Re-export for convenience
pub use db::{DatabaseConfig, create_pool, run_migrations};
pub use note_repository::SqliteNoteRepository;
pub use tag_repository::SqliteTagRepository;
pub use user_repository::SqliteUserRepository;
