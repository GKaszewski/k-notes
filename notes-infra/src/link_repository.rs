use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use notes_domain::entities::NoteLink;
use notes_domain::errors::{DomainError, DomainResult};
use notes_domain::ports::LinkRepository;

pub struct SqliteLinkRepository {
    pool: SqlitePool,
}

impl SqliteLinkRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LinkRepository for SqliteLinkRepository {
    async fn save_links(&self, links: &[NoteLink]) -> DomainResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        for link in links {
            let source = link.source_note_id.to_string();
            let target = link.target_note_id.to_string();
            let created_at = link.created_at.to_rfc3339();

            sqlx::query(
                r#"
                INSERT INTO note_links (source_note_id, target_note_id, score, created_at)
                VALUES (?, ?, ?, ?)
                ON CONFLICT(source_note_id, target_note_id) DO UPDATE SET
                    score = excluded.score,
                    created_at = excluded.created_at
                "#,
            )
            .bind(source)
            .bind(target)
            .bind(link.score)
            .bind(created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn delete_links_for_source(&self, source_note_id: Uuid) -> DomainResult<()> {
        let source_str = source_note_id.to_string();
        sqlx::query("DELETE FROM note_links WHERE source_note_id = ?")
            .bind(source_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn get_links_for_note(&self, source_note_id: Uuid) -> DomainResult<Vec<NoteLink>> {
        let source_str = source_note_id.to_string();

        // We select links where the note is the source
        // TODO: Should we also include links where the note is the target?
        // For now, let's stick to outgoing links as defined by the service logic.
        // Actually, semantic similarity is symmetric, but we only save (A -> B) if we process A.
        // Ideally we should look for both directions or enforce symmetry.
        // Given current implementation saves A->B when A is processed, if B is processed it saves B->A.
        // So just querying source_note_id is fine if we assume all notes are processed.

        let links = sqlx::query_as::<_, SqliteNoteLink>(
            "SELECT * FROM note_links WHERE source_note_id = ? ORDER BY score DESC",
        )
        .bind(source_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(links.into_iter().map(NoteLink::from).collect())
    }
}

#[derive(sqlx::FromRow)]
struct SqliteNoteLink {
    source_note_id: String,
    target_note_id: String,
    score: f32,
    created_at: String, // Stored as ISO string
}

impl From<SqliteNoteLink> for NoteLink {
    fn from(row: SqliteNoteLink) -> Self {
        Self {
            source_note_id: Uuid::parse_str(&row.source_note_id).unwrap_or_default(),
            target_note_id: Uuid::parse_str(&row.target_note_id).unwrap_or_default(),
            score: row.score,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
        }
    }
}
