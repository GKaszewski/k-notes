use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use domain::{
    errors::DomainResult,
    note::entity::{Note, NoteId},
    smart::ports::{EmbeddingGenerator, VectorStore},
    user::entity::UserId,
};
use uuid::Uuid;

use crate::{
    notes::{commands::CreateNoteCommand, create_note},
    smart::process_note,
    test_helpers::TestContext,
};

struct FakeEmbedder;

#[async_trait]
impl EmbeddingGenerator for FakeEmbedder {
    async fn generate(&self, _text: &str) -> DomainResult<Vec<f32>> {
        Ok(vec![1.0, 0.0, 0.0])
    }
}

#[derive(Default)]
struct FakeVectorStore {
    upserted: Mutex<Vec<NoteId>>,
}

#[async_trait]
impl VectorStore for FakeVectorStore {
    async fn upsert(&self, id: &NoteId, _vector: &[f32]) -> DomainResult<()> {
        self.upserted.lock().unwrap().push(*id);
        Ok(())
    }

    async fn find_similar(
        &self,
        _vector: &[f32],
        _limit: usize,
    ) -> DomainResult<Vec<(NoteId, f32)>> {
        Ok(vec![])
    }

    async fn delete(&self, _id: &NoteId) -> DomainResult<()> {
        Ok(())
    }
}

fn ctx_with_smart() -> TestContext {
    let mut t = TestContext::new();
    let store = Arc::new(FakeVectorStore::default());
    t.ctx.services.embedding = Some(Arc::new(FakeEmbedder));
    t.ctx.services.vector_store = Some(Arc::clone(&store) as Arc<dyn VectorStore>);
    t
}

#[tokio::test]
async fn processes_note_when_smart_enabled() {
    let t = ctx_with_smart();
    let user_id = Uuid::new_v4();

    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id,
            title: None,
            content: "interesting content".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    process_note::execute(
        &t.ctx,
        note.id,
        domain::user::entity::UserId::from_uuid(user_id),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn skips_when_smart_disabled() {
    let t = TestContext::new(); // no embedding/vector_store
    let user_id = Uuid::new_v4();

    let note = create_note::execute(
        &t.ctx,
        CreateNoteCommand {
            user_id,
            title: None,
            content: "content".into(),
            color: None,
            is_pinned: false,
        },
    )
    .await
    .unwrap();

    let result = process_note::execute(
        &t.ctx,
        note.id,
        domain::user::entity::UserId::from_uuid(user_id),
    )
    .await;

    assert!(result.is_ok());
}
