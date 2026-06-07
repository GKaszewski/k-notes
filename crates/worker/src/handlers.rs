use async_trait::async_trait;

use application::{
    context::AppContext,
    smart::{delete_vectors, process_note},
};
use domain::{
    errors::DomainError,
    events::{DomainEvent, EventHandler},
};

/// Routes domain events to application use cases.
/// Smart feature use cases are skipped when the adapters are not configured
/// (embedding and vector_store are None in AppContext).
pub struct NoteEventHandler {
    ctx: AppContext,
}

impl NoteEventHandler {
    pub fn new(ctx: AppContext) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl EventHandler for NoteEventHandler {
    async fn handle(&self, event: &DomainEvent) -> Result<(), DomainError> {
        match event {
            DomainEvent::NoteCreated { note_id, user_id }
            | DomainEvent::NoteUpdated { note_id, user_id } => {
                process_note::execute(&self.ctx, *note_id, *user_id).await
            }
            DomainEvent::NoteDeleted { note_id, .. } => {
                delete_vectors::execute(&self.ctx, *note_id).await
            }
        }
    }
}
