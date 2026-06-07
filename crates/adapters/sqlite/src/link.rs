use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};

use domain::{
    errors::{DomainError, DomainResult},
    note::{
        entity::{NoteId, NoteLink},
        ports::LinkRepository,
    },
};

use crate::db::RepoExt;

pub struct SqliteLinkRepository {
    pool: SqlitePool,
}

impl SqliteLinkRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct LinkRow {
    source_note_id: String,
    target_note_id: String,
    score: f32,
    created_at: String,
}

impl TryFrom<LinkRow> for NoteLink {
    type Error = DomainError;

    fn try_from(row: LinkRow) -> Result<Self, Self::Error> {
        let source_id = NoteId::from_uuid(
            uuid::Uuid::parse_str(&row.source_note_id)
                .map_err(|e| DomainError::Repository(format!("invalid source uuid: {e}")))?,
        );
        let target_id = NoteId::from_uuid(
            uuid::Uuid::parse_str(&row.target_note_id)
                .map_err(|e| DomainError::Repository(format!("invalid target uuid: {e}")))?,
        );
        let created_at = crate::db::parse_dt(&row.created_at)?;
        Ok(NoteLink {
            source_id,
            target_id,
            score: row.score,
            created_at,
        })
    }
}

#[async_trait]
impl LinkRepository for SqliteLinkRepository {
    async fn save_links(&self, links: &[NoteLink]) -> DomainResult<()> {
        let mut tx = self.pool.begin().await.repo()?;

        for link in links {
            sqlx::query(
                r#"
                INSERT INTO note_links (source_note_id, target_note_id, score, created_at)
                VALUES (?, ?, ?, ?)
                ON CONFLICT(source_note_id, target_note_id) DO UPDATE SET
                    score = excluded.score,
                    created_at = excluded.created_at
                "#,
            )
            .bind(link.source_id.as_uuid().to_string())
            .bind(link.target_id.as_uuid().to_string())
            .bind(link.score)
            .bind(link.created_at.to_rfc3339())
            .execute(&mut *tx)
            .await
            .repo()?;
        }

        tx.commit().await.repo()
    }

    async fn delete_for_source(&self, source_id: &NoteId) -> DomainResult<()> {
        sqlx::query("DELETE FROM note_links WHERE source_note_id = ?")
            .bind(source_id.as_uuid().to_string())
            .execute(&self.pool)
            .await
            .repo()
            .map(|_| ())
    }

    async fn find_for_note(&self, note_id: &NoteId) -> DomainResult<Vec<NoteLink>> {
        sqlx::query_as::<_, LinkRow>(
            "SELECT source_note_id, target_note_id, score, created_at \
             FROM note_links WHERE source_note_id = ? ORDER BY score DESC",
        )
        .bind(note_id.as_uuid().to_string())
        .fetch_all(&self.pool)
        .await
        .repo()?
        .into_iter()
        .map(NoteLink::try_from)
        .collect()
    }
}
