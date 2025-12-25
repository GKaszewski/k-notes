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
#[cfg(feature = "smart-features")]
pub mod embeddings;
pub mod factory;
#[cfg(feature = "sqlite")]
pub mod link_repository;
#[cfg(feature = "sqlite")]
pub mod note_repository;
pub mod session_store;
#[cfg(feature = "sqlite")]
pub mod tag_repository;
#[cfg(feature = "sqlite")]
pub mod user_repository;
#[cfg(feature = "smart-features")]
pub mod vector;

// Re-export for convenience
#[cfg(feature = "sqlite")]
pub use db::create_pool;
pub use db::{DatabaseConfig, run_migrations};
#[cfg(feature = "sqlite")]
pub use link_repository::SqliteLinkRepository;
#[cfg(feature = "sqlite")]
pub use note_repository::SqliteNoteRepository;
#[cfg(feature = "sqlite")]
pub use tag_repository::SqliteTagRepository;
#[cfg(feature = "sqlite")]
pub use user_repository::SqliteUserRepository;
