pub mod consumer;
pub mod publisher;

use std::time::Duration;

use async_nats::jetstream::{self, consumer as nats_consumer, consumer::pull};

use crate::{consumer::NatsEventConsumer, publisher::NatsEventPublisher};

// ── Subject routing ───────────────────────────────────────────────────────────

pub(crate) fn subject_for(event: &domain::events::DomainEvent) -> &'static str {
    use domain::events::DomainEvent;
    match event {
        DomainEvent::NoteCreated { .. } => "knotes.note.created",
        DomainEvent::NoteUpdated { .. } => "knotes.note.updated",
        DomainEvent::NoteDeleted { .. } => "knotes.note.deleted",
    }
}

pub(crate) const SUBSCRIBE_SUBJECT: &str = "knotes.note.>";

// ── Config ────────────────────────────────────────────────────────────────────

/// Configuration for the JetStream stream and durable pull consumer.
///
/// **Dead-letter queue**: after `max_deliver` failed attempts NATS stops
/// redelivering and publishes an advisory to
/// `$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES.{stream}.{consumer}`.
/// Subscribe to those with a monitoring consumer or NATS dashboard to
/// observe dead messages.
#[derive(Debug, Clone)]
pub struct JetStreamConfig {
    /// Name of the JetStream stream (created on first use if absent).
    pub stream_name: String,
    /// Durable consumer name — survives worker restarts.
    pub consumer_name: String,
    /// Maximum delivery attempts before the message is considered dead.
    pub max_deliver: i64,
    /// How long JetStream waits for an ack before redelivering.
    pub ack_wait: Duration,
}

impl Default for JetStreamConfig {
    fn default() -> Self {
        Self {
            stream_name: "KNOTES".into(),
            consumer_name: "knotes-worker".into(),
            max_deliver: 5,
            ack_wait: Duration::from_secs(30),
        }
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────

/// Connect to NATS and initialise both the publisher and consumer.
/// Creates the JetStream stream and durable pull consumer if they do not exist.
pub async fn setup(
    url: &str,
    config: JetStreamConfig,
) -> Result<(NatsEventPublisher, NatsEventConsumer), Box<dyn std::error::Error + Send + Sync>> {
    let client = async_nats::connect(url).await?;
    let js = jetstream::new(client);

    let stream = js
        .get_or_create_stream(jetstream::stream::Config {
            name: config.stream_name.clone(),
            subjects: vec![SUBSCRIBE_SUBJECT.into()],
            ..Default::default()
        })
        .await?;

    let nats_consumer: nats_consumer::Consumer<pull::Config> = stream
        .get_or_create_consumer(
            &config.consumer_name,
            pull::Config {
                durable_name: Some(config.consumer_name.clone()),
                ack_policy: jetstream::consumer::AckPolicy::Explicit,
                max_deliver: config.max_deliver,
                ack_wait: config.ack_wait,
                filter_subject: SUBSCRIBE_SUBJECT.into(),
                ..Default::default()
            },
        )
        .await?;

    Ok((
        NatsEventPublisher::new(js),
        NatsEventConsumer::new(nats_consumer),
    ))
}
