use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::BoxStream;
use tokio::sync::broadcast;

use domain::{
    errors::DomainError,
    events::{DomainEvent, EventConsumer, EventEnvelope, EventPublisher},
};

const CHANNEL_CAPACITY: usize = 256;

/// Shared in-memory event bus backed by a tokio broadcast channel.
/// Create one bus, then hand out publisher and consumer handles from it.
pub struct MemoryEventBus {
    sender: broadcast::Sender<DomainEvent>,
}

impl MemoryEventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    pub fn publisher(&self) -> Arc<MemoryEventPublisher> {
        Arc::new(MemoryEventPublisher {
            sender: self.sender.clone(),
        })
    }

    pub fn consumer(&self) -> Arc<MemoryEventConsumer> {
        Arc::new(MemoryEventConsumer {
            sender: self.sender.clone(),
        })
    }
}

impl Default for MemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MemoryEventPublisher {
    sender: broadcast::Sender<DomainEvent>,
}

#[async_trait]
impl EventPublisher for MemoryEventPublisher {
    async fn publish(&self, event: &DomainEvent) -> Result<(), DomainError> {
        // send() only fails when there are no receivers; that is fine in dev/test.
        let _ = self.sender.send(event.clone());
        Ok(())
    }
}

pub struct MemoryEventConsumer {
    sender: broadcast::Sender<DomainEvent>,
}

impl EventConsumer for MemoryEventConsumer {
    fn consume(&self) -> BoxStream<'_, Result<EventEnvelope, DomainError>> {
        let rx = self.sender.subscribe();

        Box::pin(futures::stream::unfold(rx, |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        let envelope = EventEnvelope::noop(event);
                        return Some((Ok(envelope), rx));
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("memory event bus: consumer lagged, skipped {n} messages");
                    }
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        }))
    }
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
