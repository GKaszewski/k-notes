use futures::{future::BoxFuture, stream::BoxStream};

use crate::{errors::DomainError, note::entity::NoteId, user::entity::UserId};

#[derive(Debug, Clone)]
pub enum DomainEvent {
    NoteCreated { note_id: NoteId, user_id: UserId },
    NoteUpdated { note_id: NoteId, user_id: UserId },
    NoteDeleted { note_id: NoteId, user_id: UserId },
}

type AckFn = Box<dyn FnOnce() -> BoxFuture<'static, Result<(), DomainError>> + Send>;

pub struct EventEnvelope {
    pub event: DomainEvent,
    ack_fn: AckFn,
    nack_fn: AckFn,
}

impl EventEnvelope {
    pub fn new(
        event: DomainEvent,
        ack_fn: impl FnOnce() -> BoxFuture<'static, Result<(), DomainError>> + Send + 'static,
        nack_fn: impl FnOnce() -> BoxFuture<'static, Result<(), DomainError>> + Send + 'static,
    ) -> Self {
        Self {
            event,
            ack_fn: Box::new(ack_fn),
            nack_fn: Box::new(nack_fn),
        }
    }

    /// Both ack and nack are no-ops. For in-memory and test consumers.
    pub fn noop(event: DomainEvent) -> Self {
        Self::new(
            event,
            || Box::pin(async { Ok(()) }),
            || Box::pin(async { Ok(()) }),
        )
    }

    pub async fn ack(self) -> Result<(), DomainError> {
        (self.ack_fn)().await
    }

    /// Signal that processing failed. The transport decides whether to redeliver
    /// (JetStream: redeliver up to max_deliver times; in-memory: no-op).
    pub async fn nack(self) -> Result<(), DomainError> {
        (self.nack_fn)().await
    }
}

#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &DomainEvent) -> Result<(), DomainError>;
}

pub trait EventConsumer: Send + Sync {
    fn consume(&self) -> BoxStream<'_, Result<EventEnvelope, DomainError>>;
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent) -> Result<(), DomainError>;
}
