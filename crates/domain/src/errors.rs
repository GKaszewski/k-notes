use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("repository: {0}")]
    Repository(String),
    #[error("infrastructure: {0}")]
    Infrastructure(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
