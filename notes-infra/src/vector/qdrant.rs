use async_trait::async_trait;
use notes_domain::errors::{DomainError, DomainResult};
use notes_domain::ports::VectorStore;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
    Value, VectorParamsBuilder,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct QdrantVectorAdapter {
    client: Arc<Qdrant>,
    collection_name: String,
}

impl QdrantVectorAdapter {
    pub fn new(url: &str, collection_name: &str) -> DomainResult<Self> {
        let client = Qdrant::from_url(url).build().map_err(|e| {
            DomainError::InfrastructureError(format!("Failed to create Qdrant client: {}", e))
        })?;

        Ok(Self {
            client: Arc::new(client),
            collection_name: collection_name.to_string(),
        })
    }

    pub async fn create_collection_if_not_exists(&self) -> DomainResult<()> {
        if !self
            .client
            .collection_exists(&self.collection_name)
            .await
            .map_err(|e| {
                DomainError::InfrastructureError(format!(
                    "Failed to check collection existence: {}",
                    e
                ))
            })?
        {
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(self.collection_name.clone())
                        .vectors_config(VectorParamsBuilder::new(384, Distance::Cosine)),
                )
                .await
                .map_err(|e| {
                    DomainError::InfrastructureError(format!("Failed to create collection: {}", e))
                })?;
        }
        Ok(())
    }
}

#[async_trait]
impl VectorStore for QdrantVectorAdapter {
    async fn upsert(&self, id: Uuid, vector: &[f32]) -> DomainResult<()> {
        let payload: HashMap<String, Value> = HashMap::new();

        let point = PointStruct::new(id.to_string(), vector.to_vec(), payload);

        let upsert_points = UpsertPointsBuilder::new(self.collection_name.clone(), vec![point]);

        self.client
            .upsert_points(upsert_points)
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("Qdrant upsert error: {}", e)))?;

        Ok(())
    }

    async fn find_similar(&self, vector: &[f32], limit: usize) -> DomainResult<Vec<(Uuid, f32)>> {
        let search_points =
            SearchPointsBuilder::new(self.collection_name.clone(), vector.to_vec(), limit as u64)
                .with_payload(true);

        let search_result = self
            .client
            .search_points(search_points)
            .await
            .map_err(|e| DomainError::InfrastructureError(format!("Qdrant search error: {}", e)))?;

        let results = search_result
            .result
            .into_iter()
            .filter_map(|point| {
                let id = point.id?;
                let uuid_str = match id.point_id_options? {
                    qdrant_client::qdrant::point_id::PointIdOptions::Uuid(u) => u,
                    _ => return None,
                };

                let uuid = Uuid::parse_str(&uuid_str).ok()?;
                Some((uuid, point.score))
            })
            .collect();

        Ok(results)
    }
}
