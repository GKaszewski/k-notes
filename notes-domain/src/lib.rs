//! K-Notes Domain Layer
//!
//! This crate contains pure domain logic with no I/O dependencies.
//! It follows hexagonal architecture principles where:
//!
//! - **Entities**: Core business objects (Note, Tag, User)
//! - **Errors**: Domain-specific error types
//! - **Repositories**: Port traits defining data access interfaces
//! - **Services**: Use cases orchestrating business logic

pub mod entities;
pub mod errors;
pub mod repositories;
pub mod services;

// Re-export commonly used types at crate root
pub use entities::{MAX_TAGS_PER_NOTE, Note, NoteFilter, NoteVersion, Tag, User};
pub use errors::{DomainError, DomainResult};
pub use repositories::{NoteRepository, TagRepository, UserRepository};
pub use services::{CreateNoteRequest, NoteService, TagService, UpdateNoteRequest, UserService};
