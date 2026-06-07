use std::{sync::Arc, time::Duration};

use async_nats::jetstream::{
    AckKind,
    consumer::{self, pull},
};
use futures::{StreamExt, future::BoxFuture, stream::BoxStream};

use domain::{
    errors::DomainError,
    events::{DomainEvent, EventConsumer, EventEnvelope},
};
use event_payload::EventPayload;

pub struct NatsEventConsumer {
    consumer: Arc<consumer::Consumer<pull::Config>>,
}

impl NatsEventConsumer {
    pub(crate) fn new(consumer: consumer::Consumer<pull::Config>) -> Self {
        Self {
            consumer: Arc::new(consumer),
        }
    }
}

impl EventConsumer for NatsEventConsumer {
    fn consume(&self) -> BoxStream<'_, Result<EventEnvelope, DomainError>> {
        let consumer = Arc::clone(&self.consumer);

        Box::pin(async_stream::stream! {
            let mut messages = match consumer.messages().await {
                Ok(m) => m,
                Err(e) => {
                    yield Err(DomainError::Infrastructure(
                        format!("failed to open jetstream message stream: {e}")
                    ));
                    return;
                }
            };

            while let Some(result) = messages.next().await {
                let msg = match result {
                    Ok(m) => m,
                    Err(e) => {
                        yield Err(DomainError::Infrastructure(e.to_string()));
                        continue;
                    }
                };

                // Malformed messages are acked immediately to prevent infinite
                // redelivery of poison payloads that can never be processed.
                let payload = match EventPayload::from_json(&msg.payload) {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!("unprocessable message payload, acking to discard: {e}");
                        let _ = msg.ack().await;
                        continue;
                    }
                };

                let event = match DomainEvent::try_from(payload) {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("invalid event payload, acking to discard: {e}");
                        let _ = msg.ack().await;
                        continue;
                    }
                };

                let delivered = msg.info().map(|i| i.delivered).unwrap_or(1);
                let nack_delay = backoff(delivered);

                let msg = Arc::new(msg);
                let ack_msg = Arc::clone(&msg);
                let nack_msg = Arc::clone(&msg);

                yield Ok(EventEnvelope::new(
                    event,
                    move || -> BoxFuture<'static, _> {
                        Box::pin(async move {
                            ack_msg.ack().await.map_err(|e| {
                                DomainError::Infrastructure(format!("nats ack failed: {e}"))
                            })
                        })
                    },
                    move || -> BoxFuture<'static, _> {
                        Box::pin(async move {
                            nack_msg.ack_with(AckKind::Nak(Some(nack_delay))).await.map_err(|e| {
                                DomainError::Infrastructure(format!("nats nack failed: {e}"))
                            })
                        })
                    },
                ));
            }
        })
    }
}

/// Exponential backoff capped at 5 minutes: 1s → 5s → 25s → 125s → 300s.
fn backoff(delivered: i64) -> Duration {
    let exp = delivered.saturating_sub(1) as u32;
    Duration::from_secs(5u64.saturating_pow(exp).min(300))
}
