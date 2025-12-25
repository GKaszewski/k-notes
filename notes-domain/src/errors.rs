//! Domain errors for K-Notes
//!
//! Uses `thiserror` for ergonomic error definitions.
//! These errors represent domain-level failures and will be mapped
//! to HTTP status codes in the API layer.

use thiserror::Error;
use uuid::Uuid;

use crate::entities::MAX_TAGS_PER_NOTE;

/// Domain-level errors for K-Notes operations
#[derive(Debug, Error)]
pub enum DomainError {
    /// The requested note was not found
    #[error("Note not found: {0}")]
    NoteNotFound(Uuid),

    /// The requested user was not found
    #[error("User not found: {0}")]
    UserNotFound(Uuid),

    /// The requested tag was not found
    #[error("Tag not found: {0}")]
    TagNotFound(Uuid),

    /// User with this email/subject already exists
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),

    /// Tag with this name already exists for the user
    #[error("Tag already exists: {0}")]
    TagAlreadyExists(String),

    /// Attempted to add too many tags to a note
    #[error("Tag limit exceeded: maximum {max} tags allowed, note has {current}")]
    TagLimitExceeded { max: usize, current: usize },

    /// A validation error occurred
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// User is not authorized to perform this action
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// A repository/infrastructure error occurred
    #[error("Repository error: {0}")]
    RepositoryError(String),

    /// An infrastructure adapter error occurred
    #[error("Infrastructure error: {0}")]
    InfrastructureError(String),
}

impl DomainError {
    /// Create a tag limit exceeded error with the current count
    pub fn tag_limit_exceeded(current: usize) -> Self {
        Self::TagLimitExceeded {
            max: MAX_TAGS_PER_NOTE,
            current,
        }
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    /// Create an unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized(message.into())
    }

    /// Check if this error indicates a "not found" condition
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            DomainError::NoteNotFound(_)
                | DomainError::UserNotFound(_)
                | DomainError::TagNotFound(_)
        )
    }

    /// Check if this error indicates a conflict (already exists)
    pub fn is_conflict(&self) -> bool {
        matches!(
            self,
            DomainError::UserAlreadyExists(_) | DomainError::TagAlreadyExists(_)
        )
    }
}

/// Result type alias for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_limit_exceeded_uses_constant() {
        let error = DomainError::tag_limit_exceeded(15);

        if let DomainError::TagLimitExceeded { max, current } = error {
            assert_eq!(max, MAX_TAGS_PER_NOTE);
            assert_eq!(current, 15);
        } else {
            panic!("Expected TagLimitExceeded error");
        }
    }

    #[test]
    fn test_error_display_messages() {
        let note_id = Uuid::new_v4();
        let error = DomainError::NoteNotFound(note_id);
        assert!(error.to_string().contains(&note_id.to_string()));

        let error = DomainError::validation("Title cannot be empty");
        assert_eq!(error.to_string(), "Validation error: Title cannot be empty");
    }

    #[test]
    fn test_is_not_found() {
        assert!(DomainError::NoteNotFound(Uuid::new_v4()).is_not_found());
        assert!(DomainError::UserNotFound(Uuid::new_v4()).is_not_found());
        assert!(DomainError::TagNotFound(Uuid::new_v4()).is_not_found());
        assert!(!DomainError::validation("test").is_not_found());
    }

    #[test]
    fn test_is_conflict() {
        assert!(DomainError::UserAlreadyExists("test@example.com".into()).is_conflict());
        assert!(DomainError::TagAlreadyExists("work".into()).is_conflict());
        assert!(!DomainError::NoteNotFound(Uuid::new_v4()).is_conflict());
    }
}
