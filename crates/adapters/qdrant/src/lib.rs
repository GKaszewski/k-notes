use async_trait::async_trait;
use qdrant_client::{
    Qdrant, QdrantError,
    qdrant::{
        CreateCollectionBuilder, DeletePointsBuilder, Distance, PointId, PointStruct,
        PointsIdsList, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
        point_id::PointIdOptions,
    },
};
use uuid::Uuid;

use domain::{
    errors::{DomainError, DomainResult},
    note::entity::NoteId,
    smart::ports::VectorStore,
};

pub struct QdrantConfig {
    pub url: String,
    pub collection: String,
    /// Dimensionality of the vectors stored in this collection.
    /// Must match the output size of the embedding model (e.g. 384 for AllMiniLML6V2).
    pub vector_size: u64,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".into(),
            collection: "notes".into(),
            vector_size: 384,
        }
    }
}

pub struct QdrantVectorStore {
    client: Qdrant,
    collection: String,
}

impl QdrantVectorStore {
    pub fn new(config: QdrantConfig) -> Result<Self, Box<QdrantError>> {
        let client = Qdrant::from_url(&config.url).build().map_err(Box::new)?;
        Ok(Self {
            client,
            collection: config.collection,
        })
    }

    /// Ensure the collection exists. Call once during startup before accepting requests.
    pub async fn init(&self, vector_size: u64) -> DomainResult<()> {
        if self
            .client
            .collection_exists(&self.collection)
            .await
            .map_err(qdrant_err)?
        {
            return Ok(());
        }

        self.client
            .create_collection(
                CreateCollectionBuilder::new(&self.collection)
                    .vectors_config(VectorParamsBuilder::new(vector_size, Distance::Cosine)),
            )
            .await
            .map_err(qdrant_err)?;

        tracing::info!(collection = %self.collection, "qdrant collection created");
        Ok(())
    }
}

#[async_trait]
impl VectorStore for QdrantVectorStore {
    async fn upsert(&self, id: &NoteId, vector: &[f32]) -> DomainResult<()> {
        let point = PointStruct::new(
            uuid_to_point_id(id.as_uuid()),
            vector.to_vec(),
            qdrant_client::Payload::default(),
        );

        self.client
            .upsert_points(UpsertPointsBuilder::new(&self.collection, vec![point]))
            .await
            .map_err(qdrant_err)
            .map(|_| ())
    }

    async fn find_similar(&self, vector: &[f32], limit: usize) -> DomainResult<Vec<(NoteId, f32)>> {
        let response = self
            .client
            .search_points(
                SearchPointsBuilder::new(&self.collection, vector.to_vec(), limit as u64)
                    .with_payload(false),
            )
            .await
            .map_err(qdrant_err)?;

        response
            .result
            .into_iter()
            .filter_map(|scored| {
                let uuid_str = match scored.id?.point_id_options? {
                    PointIdOptions::Uuid(s) => s,
                    _ => return None,
                };
                let uuid = Uuid::parse_str(&uuid_str).ok()?;
                Some(Ok((NoteId::from_uuid(uuid), scored.score)))
            })
            .collect()
    }

    async fn delete(&self, id: &NoteId) -> DomainResult<()> {
        self.client
            .delete_points(
                DeletePointsBuilder::new(&self.collection).points(PointsIdsList {
                    ids: vec![uuid_to_point_id(id.as_uuid())],
                }),
            )
            .await
            .map_err(qdrant_err)
            .map(|_| ())
    }
}

fn uuid_to_point_id(uuid: Uuid) -> PointId {
    PointId {
        point_id_options: Some(PointIdOptions::Uuid(uuid.to_string())),
    }
}

fn qdrant_err(e: QdrantError) -> DomainError {
    DomainError::Infrastructure(format!("qdrant: {e}"))
}

#[cfg(test)]
#[path = "tests/lib.rs"]
mod tests;
