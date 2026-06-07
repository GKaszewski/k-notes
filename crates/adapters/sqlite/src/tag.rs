use async_trait::async_trait;
use sqlx::{FromRow, SqlitePool};

use domain::{
    errors::{DomainError, DomainResult},
    note::entity::NoteId,
    tag::{
        entity::{Tag, TagId},
        ports::TagRepository,
        value_objects::TagName,
    },
    user::entity::UserId,
};

use crate::db::RepoExt;

pub struct SqliteTagRepository {
    pool: SqlitePool,
}

impl SqliteTagRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct TagRow {
    id: String,
    name: String,
    user_id: String,
}

impl TryFrom<TagRow> for Tag {
    type Error = DomainError;

    fn try_from(row: TagRow) -> Result<Self, Self::Error> {
        let id = TagId::from_uuid(
            uuid::Uuid::parse_str(&row.id)
                .map_err(|e| DomainError::Repository(format!("invalid tag uuid: {e}")))?,
        );
        let user_id = UserId::from_uuid(
            uuid::Uuid::parse_str(&row.user_id)
                .map_err(|e| DomainError::Repository(format!("invalid user uuid: {e}")))?,
        );
        let name = TagName::new(row.name)?;
        Ok(Tag::from_row(id, name, user_id))
    }
}

#[async_trait]
impl TagRepository for SqliteTagRepository {
    async fn find_by_id(&self, id: &TagId) -> DomainResult<Option<Tag>> {
        sqlx::query_as::<_, TagRow>("SELECT id, name, user_id FROM tags WHERE id = ?")
            .bind(id.as_uuid().to_string())
            .fetch_optional(&self.pool)
            .await
            .repo()?
            .map(Tag::try_from)
            .transpose()
    }

    async fn find_by_user(&self, user_id: &UserId) -> DomainResult<Vec<Tag>> {
        sqlx::query_as::<_, TagRow>(
            "SELECT id, name, user_id FROM tags WHERE user_id = ? ORDER BY name",
        )
        .bind(user_id.as_uuid().to_string())
        .fetch_all(&self.pool)
        .await
        .repo()?
        .into_iter()
        .map(Tag::try_from)
        .collect()
    }

    async fn find_by_name(&self, user_id: &UserId, name: &TagName) -> DomainResult<Option<Tag>> {
        sqlx::query_as::<_, TagRow>(
            "SELECT id, name, user_id FROM tags WHERE user_id = ? AND name = ?",
        )
        .bind(user_id.as_uuid().to_string())
        .bind(name.as_ref())
        .fetch_optional(&self.pool)
        .await
        .repo()?
        .map(Tag::try_from)
        .transpose()
    }

    async fn find_by_note(&self, note_id: &NoteId) -> DomainResult<Vec<Tag>> {
        sqlx::query_as::<_, TagRow>(
            r#"
            SELECT t.id, t.name, t.user_id
            FROM tags t
            INNER JOIN note_tags nt ON t.id = nt.tag_id
            WHERE nt.note_id = ?
            ORDER BY t.name
            "#,
        )
        .bind(note_id.as_uuid().to_string())
        .fetch_all(&self.pool)
        .await
        .repo()?
        .into_iter()
        .map(Tag::try_from)
        .collect()
    }

    async fn save(&self, tag: &Tag) -> DomainResult<()> {
        sqlx::query(
            r#"
            INSERT INTO tags (id, name, user_id)
            VALUES (?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET name = excluded.name
            "#,
        )
        .bind(tag.id.as_uuid().to_string())
        .bind(tag.name.as_ref())
        .bind(tag.user_id.as_uuid().to_string())
        .execute(&self.pool)
        .await
        .repo()
        .map(|_| ())
    }

    async fn delete(&self, id: &TagId) -> DomainResult<()> {
        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(id.as_uuid().to_string())
            .execute(&self.pool)
            .await
            .repo()
            .map(|_| ())
    }

    async fn add_to_note(&self, tag_id: &TagId, note_id: &NoteId) -> DomainResult<()> {
        sqlx::query("INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?, ?)")
            .bind(note_id.as_uuid().to_string())
            .bind(tag_id.as_uuid().to_string())
            .execute(&self.pool)
            .await
            .repo()
            .map(|_| ())
    }

    async fn remove_from_note(&self, tag_id: &TagId, note_id: &NoteId) -> DomainResult<()> {
        sqlx::query("DELETE FROM note_tags WHERE note_id = ? AND tag_id = ?")
            .bind(note_id.as_uuid().to_string())
            .bind(tag_id.as_uuid().to_string())
            .execute(&self.pool)
            .await
            .repo()
            .map(|_| ())
    }
}

#[cfg(test)]
#[path = "tests/tag.rs"]
mod tests;
