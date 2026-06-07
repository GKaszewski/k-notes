use crate::{FastEmbedConfig, FastEmbedGenerator};
use domain::smart::ports::EmbeddingGenerator;
use fastembed::EmbeddingModel;

/// Downloads the model on first run (~90 MB). Run with:
///   cargo test -p fastembed-adapter -- --ignored
#[tokio::test]
#[ignore]
async fn generates_embedding_with_correct_dimension() {
    let generator =
        FastEmbedGenerator::new(FastEmbedConfig::with_model(EmbeddingModel::AllMiniLML6V2))
            .expect("model init failed");

    let embedding = generator.generate("hello world").await.unwrap();

    // AllMiniLML6V2 produces 384-dimensional vectors.
    assert_eq!(embedding.len(), 384);
    // Sanity: values are in a reasonable range.
    assert!(embedding.iter().all(|v| v.is_finite()));
}

#[tokio::test]
#[ignore]
async fn different_texts_produce_different_embeddings() {
    let generator =
        FastEmbedGenerator::new(FastEmbedConfig::with_model(EmbeddingModel::AllMiniLML6V2))
            .expect("model init failed");

    let a = generator.generate("cats").await.unwrap();
    let b = generator.generate("quantum mechanics").await.unwrap();

    assert_ne!(a, b);
}
