use async_nats::jetstream;
use async_trait::async_trait;

use domain::{
    errors::DomainError,
    events::{DomainEvent, EventPublisher},
};
use event_payload::EventPayload;

use crate::subject_for;

pub struct NatsEventPublisher {
    js: jetstream::Context,
}

impl NatsEventPublisher {
    pub(crate) fn new(js: jetstream::Context) -> Self {
        Self { js }
    }
}

#[async_trait]
impl EventPublisher for NatsEventPublisher {
    async fn publish(&self, event: &DomainEvent) -> Result<(), DomainError> {
        let bytes = EventPayload::from(event).to_json()?;
        self.js
            .publish(subject_for(event), bytes.into())
            .await
            .map_err(|e| DomainError::Infrastructure(format!("nats publish failed: {e}")))?
            .await
            .map_err(|e| DomainError::Infrastructure(format!("nats publish ack failed: {e}")))?;
        Ok(())
    }
}
