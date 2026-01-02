//! NATS message broker adapter
//!
//! Implements the `MessageBroker` port for NATS messaging.

use std::pin::Pin;

use async_trait::async_trait;
use futures_util::StreamExt;
use k_core::broker::{MessageBroker as CoreBroker, nats::NatsBroker};
use notes_domain::{DomainError, DomainResult, MessageBroker, Note};

pub struct NatsMessageBroker {
    inner: NatsBroker,
}

impl NatsMessageBroker {
    pub fn new(broker: NatsBroker) -> Self {
        Self { inner: broker }
    }
}

#[async_trait]
impl MessageBroker for NatsMessageBroker {
    async fn publish_note_updated(&self, note: &Note) -> DomainResult<()> {
        let payload = serde_json::to_vec(note).map_err(|e| {
            DomainError::RepositoryError(format!("Failed to serialize note: {}", e))
        })?;

        self.inner
            .publish("notes.updated", payload.into())
            .await
            .map_err(|e| DomainError::RepositoryError(format!("Failed to publish event: {}", e)))?;

        Ok(())
    }

    async fn subscribe_note_updates(
        &self,
    ) -> DomainResult<Pin<Box<dyn futures_core::Stream<Item = Note> + Send>>> {
        let stream =
            self.inner.subscribe("notes.updated").await.map_err(|e| {
                DomainError::RepositoryError(format!("Broker subscribe error: {}", e))
            })?;

        // Map generic bytes back to Domain Note
        let note_stream = stream.filter_map(|bytes| async move {
            match serde_json::from_slice::<Note>(&bytes) {
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
