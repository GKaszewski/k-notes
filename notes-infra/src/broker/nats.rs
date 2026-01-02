//! NATS message broker adapter
//!
//! Implements the `MessageBroker` port for NATS messaging.

use std::pin::Pin;

use async_trait::async_trait;
use futures_util::StreamExt;
use notes_domain::{DomainError, DomainResult, MessageBroker, Note};

/// NATS adapter implementing the MessageBroker port.
pub struct NatsMessageBroker {
    client: async_nats::Client,
}

impl NatsMessageBroker {
    /// Create a new NATS message broker by connecting to the given URL.
    pub async fn connect(url: &str) -> Result<Self, async_nats::ConnectError> {
        let client = async_nats::connect(url).await?;
        Ok(Self { client })
    }

    /// Create a NATS message broker from an existing client.
    pub fn from_client(client: async_nats::Client) -> Self {
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

    async fn subscribe_note_updates(
        &self,
    ) -> DomainResult<Pin<Box<dyn futures_core::Stream<Item = Note> + Send>>> {
        let subscriber = self
            .client
            .subscribe("notes.updated")
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to subscribe: {}", e)))?;

        // Transform the NATS message stream into a Note stream
        let note_stream = subscriber.filter_map(|msg| async move {
            match serde_json::from_slice::<Note>(&msg.payload) {
                Ok(note) => Some(note),
                Err(e) => {
                    tracing::warn!("Failed to deserialize note from message: {}", e);
                    None
                }
            }
        });

        Ok(Box::pin(note_stream))
    }
}
