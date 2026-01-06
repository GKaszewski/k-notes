//! K-Notes Domain Layer
//!
//! This crate contains pure domain logic with no I/O dependencies.
//! It follows hexagonal architecture principles where:
//!
//! - **Entities**: Core business objects (Note, Tag, User)
//! - **Errors**: Domain-specific error types
//! - **Repositories**: Port traits defining data access interfaces
//! - **Services**: Use cases orchestrating business logic
//! - **Value Objects**: Validated newtypes for domain primitives

pub mod entities;
pub mod errors;
pub mod ports;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-export commonly used types at crate root
pub use entities::*;
pub use errors::{DomainError, DomainResult};
pub use ports::*;
pub use repositories::*;
pub use services::*;
pub use value_objects::*;
