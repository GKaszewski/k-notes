use async_trait::async_trait;
use uuid::Uuid;

use crate::entities::{Note, NoteLink};
use crate::errors::DomainResult;

/// Defines how to generate vector embeddings from text.
#[async_trait]
pub trait EmbeddingGenerator: Send + Sync {
    /// Generate a vector embedding for the given text.
    async fn generate_embedding(&self, text: &str) -> DomainResult<Vec<f32>>;
}

/// Defines how to store and retrieve vectors.
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Upsert a vector for a given note ID.
    async fn upsert(&self, id: Uuid, vector: &[f32]) -> DomainResult<()>;

    /// Find similar items to the given vector.
    /// Returns a list of (NoteID, Score) tuples.
    async fn find_similar(&self, vector: &[f32], limit: usize) -> DomainResult<Vec<(Uuid, f32)>>;
}

/// Defines how to persist note links.
#[async_trait]
pub trait LinkRepository: Send + Sync {
    /// Save a batch of generated links.
    async fn save_links(&self, links: &[NoteLink]) -> DomainResult<()>;

    /// Delete existing links for a specific source note (e.g., before regenerating).
    async fn delete_links_for_source(&self, source_note_id: Uuid) -> DomainResult<()>;

    /// Get links for a specific source note.
    async fn get_links_for_note(&self, source_note_id: Uuid) -> DomainResult<Vec<NoteLink>>;
}

/// Port for publishing domain events to a message broker.
/// Enables the Service layer to trigger background processing
/// without coupling to a specific messaging implementation.
#[async_trait]
pub trait MessageBroker: Send + Sync {
    /// Publish an event when a note is created or updated.
    async fn publish_note_updated(&self, note: &Note) -> DomainResult<()>;
}
