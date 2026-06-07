use futures::StreamExt;

use domain::{
    events::{DomainEvent, EventConsumer, EventPublisher},
    note::entity::NoteId,
    user::entity::UserId,
};

use crate::MemoryEventBus;

fn note_updated() -> DomainEvent {
    DomainEvent::NoteUpdated {
        note_id: NoteId::new(),
        user_id: UserId::new(),
    }
}

#[tokio::test]
async fn published_event_is_received_by_consumer() {
    let bus = MemoryEventBus::new();
    let publisher = bus.publisher();
    let consumer = bus.consumer();

    let event = note_updated();
    let mut stream = consumer.consume();

    publisher.publish(&event).await.unwrap();

    let envelope = stream.next().await.unwrap().unwrap();
    assert!(matches!(envelope.event, DomainEvent::NoteUpdated { .. }));
}

#[tokio::test]
async fn ack_on_memory_envelope_is_noop() {
    let bus = MemoryEventBus::new();
    let publisher = bus.publisher();
    let consumer = bus.consumer();

    // Subscribe before publishing — broadcast drops messages sent before subscribe.
    let mut stream = consumer.consume();
    publisher.publish(&note_updated()).await.unwrap();

    let envelope = stream.next().await.unwrap().unwrap();
    envelope.ack().await.unwrap();
}

#[tokio::test]
async fn multiple_consumers_each_receive_the_event() {
    let bus = MemoryEventBus::new();
    let publisher = bus.publisher();
    let c1 = bus.consumer();
    let c2 = bus.consumer();

    let mut s1 = c1.consume();
    let mut s2 = c2.consume();

    publisher.publish(&note_updated()).await.unwrap();

    assert!(matches!(
        s1.next().await.unwrap().unwrap().event,
        DomainEvent::NoteUpdated { .. }
    ));
    assert!(matches!(
        s2.next().await.unwrap().unwrap().event,
        DomainEvent::NoteUpdated { .. }
    ));
}

#[tokio::test]
async fn publish_with_no_consumer_does_not_error() {
    let bus = MemoryEventBus::new();
    let publisher = bus.publisher();
    // No consumer — publish should silently succeed.
    publisher.publish(&note_updated()).await.unwrap();
}
