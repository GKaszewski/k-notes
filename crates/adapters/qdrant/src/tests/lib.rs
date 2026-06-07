use domain::{note::entity::NoteId, smart::ports::VectorStore};

use crate::{QdrantConfig, QdrantVectorStore};

const VECTOR_SIZE: u64 = 4; // small for tests

fn test_config() -> QdrantConfig {
    QdrantConfig {
        url: "http://localhost:6334".into(),
        collection: "test-notes".into(),
        vector_size: VECTOR_SIZE,
    }
}

/// Requires a running Qdrant instance. Run with:
///   cargo test -p qdrant-adapter -- --ignored
#[tokio::test]
#[ignore]
async fn upsert_and_find_similar() {
    let store = QdrantVectorStore::new(test_config()).unwrap();
    store.init(VECTOR_SIZE).await.unwrap();

    let id = NoteId::new();
    let vector = vec![1.0f32, 0.0, 0.0, 0.0];
    store.upsert(&id, &vector).await.unwrap();

    let results = store.find_similar(&vector, 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, id);
    assert!(results[0].1 > 0.99);
}

#[tokio::test]
#[ignore]
async fn delete_removes_vector() {
    let store = QdrantVectorStore::new(test_config()).unwrap();
    store.init(VECTOR_SIZE).await.unwrap();

    let id = NoteId::new();
    store.upsert(&id, &[1.0, 0.0, 0.0, 0.0]).await.unwrap();
    store.delete(&id).await.unwrap();

    let results = store.find_similar(&[1.0, 0.0, 0.0, 0.0], 10).await.unwrap();
    assert!(!results.iter().any(|(rid, _)| rid == &id));
}
