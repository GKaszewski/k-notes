//! NATS message broker adapter for domain MessageBroker port

use async_trait::async_trait;
use notes_domain::{DomainError, DomainResult, MessageBroker, Note};

/// NATS adapter implementing the MessageBroker port
pub struct NatsMessageBroker {
    client: async_nats::Client,
}

impl NatsMessageBroker {
    pub fn new(client: async_nats::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl MessageBroker for NatsMessageBroker {
    async fn publish_note_updated(&self, note: &Note) -> DomainResult<()> {
        let payload = serde_json::to_vec(note).map_err(|e| {
            DomainError::RepositoryError(format!("Failed to serialize note: {}", e))
        })?;

        self.client
            .publish("notes.updated", payload.into())
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to publish event: {}", e)))?;

        Ok(())
    }
}
