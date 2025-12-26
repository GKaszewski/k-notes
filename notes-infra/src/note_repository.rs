//! SQLite implementation of NoteRepository with FTS5 full-text search

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use uuid::Uuid;

use notes_domain::{DomainError, DomainResult, Note, NoteFilter, NoteRepository, NoteVersion, Tag};

/// SQLite adapter for NoteRepository
pub struct SqliteNoteRepository {
    pool: SqlitePool,
}

impl SqliteNoteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

/// Row with JSON-aggregated tags for single-query fetching
#[derive(Debug, FromRow)]
struct NoteRowWithTags {
    id: String,
    user_id: String,
    title: String,
    content: String,
    color: String,
    is_pinned: i32,
    is_archived: i32,
    created_at: String,
    updated_at: String,
    tags_json: String,
}

/// Helper to parse datetime strings
fn parse_datetime(s: &str) -> Result<DateTime<Utc>, DomainError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map(|dt| dt.and_utc())
        })
        .map_err(|e| DomainError::RepositoryError(format!("Invalid datetime: {}", e)))
}

/// Helper to parse tags from JSON array
fn parse_tags_json(tags_json: &str) -> Result<Vec<Tag>, DomainError> {
    // SQLite returns [null] for LEFT JOIN with no matches
    let parsed: Vec<serde_json::Value> = serde_json::from_str(tags_json)
        .map_err(|e| DomainError::RepositoryError(format!("Failed to parse tags JSON: {}", e)))?;

    parsed
        .into_iter()
        .filter(|v| !v.is_null())
        .map(|v| {
            let id_str = v["id"]
                .as_str()
                .ok_or_else(|| DomainError::RepositoryError("Missing tag id".to_string()))?;
            let name = v["name"]
                .as_str()
                .ok_or_else(|| DomainError::RepositoryError("Missing tag name".to_string()))?;
            let user_id_str = v["user_id"]
                .as_str()
                .ok_or_else(|| DomainError::RepositoryError("Missing tag user_id".to_string()))?;

            let id = Uuid::parse_str(id_str)
                .map_err(|e| DomainError::RepositoryError(format!("Invalid tag UUID: {}", e)))?;
            let user_id = Uuid::parse_str(user_id_str)
                .map_err(|e| DomainError::RepositoryError(format!("Invalid tag user_id: {}", e)))?;

            Ok(Tag::with_id(id, name.to_string(), user_id))
        })
        .collect()
}

impl NoteRowWithTags {
    fn try_into_note(self) -> Result<Note, DomainError> {
        let id = Uuid::parse_str(&self.id)
            .map_err(|e| DomainError::RepositoryError(format!("Invalid UUID: {}", e)))?;
        let user_id = Uuid::parse_str(&self.user_id)
            .map_err(|e| DomainError::RepositoryError(format!("Invalid UUID: {}", e)))?;

        let created_at = parse_datetime(&self.created_at)?;
        let updated_at = parse_datetime(&self.updated_at)?;
        let tags = parse_tags_json(&self.tags_json)?;

        Ok(Note {
            id,
            user_id,
            title: self.title,
            content: self.content,
            color: self.color,
            is_pinned: self.is_pinned != 0,
            is_archived: self.is_archived != 0,
            created_at,
            updated_at,
            tags,
        })
    }
}

#[derive(Debug, FromRow)]
struct NoteVersionRow {
    id: String,
    note_id: String,
    title: String,
    content: String,
    created_at: String,
}

impl NoteVersionRow {
    fn try_into_version(self) -> Result<NoteVersion, DomainError> {
        let id = Uuid::parse_str(&self.id)
            .map_err(|e| DomainError::RepositoryError(format!("Invalid UUID: {}", e)))?;
        let note_id = Uuid::parse_str(&self.note_id)
            .map_err(|e| DomainError::RepositoryError(format!("Invalid UUID: {}", e)))?;

        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%d %H:%M:%S")
                    .map(|dt| dt.and_utc())
            })
            .map_err(|e| DomainError::RepositoryError(format!("Invalid datetime: {}", e)))?;

        Ok(NoteVersion {
            id,
            note_id,
            title: self.title,
            content: self.content,
            created_at,
        })
    }
}

#[async_trait]
impl NoteRepository for SqliteNoteRepository {
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Note>> {
        let id_str = id.to_string();
        let row: Option<NoteRowWithTags> = sqlx::query_as(
            r#"
            SELECT n.id, n.user_id, n.title, n.content, n.color, n.is_pinned, n.is_archived, 
                   n.created_at, n.updated_at,
                   json_group_array(
                       CASE WHEN t.id IS NOT NULL 
                       THEN json_object('id', t.id, 'name', t.name, 'user_id', t.user_id)
                       ELSE NULL END
                   ) as tags_json
            FROM notes n
            LEFT JOIN note_tags nt ON n.id = nt.note_id
            LEFT JOIN tags t ON nt.tag_id = t.id
            WHERE n.id = ?
            GROUP BY n.id
            "#,
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(row.try_into_note()?)),
            None => Ok(None),
        }
    }

    async fn find_by_user(&self, user_id: Uuid, filter: NoteFilter) -> DomainResult<Vec<Note>> {
        let user_id_str = user_id.to_string();

        // Build dynamic query using QueryBuilder for safety
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            SELECT n.id, n.user_id, n.title, n.content, n.color, n.is_pinned, n.is_archived,
                   n.created_at, n.updated_at,
                   json_group_array(
                       CASE WHEN t.id IS NOT NULL
                       THEN json_object('id', t.id, 'name', t.name, 'user_id', t.user_id)
                       ELSE NULL END
                   ) as tags_json
            FROM notes n
            LEFT JOIN note_tags nt ON n.id = nt.note_id
            LEFT JOIN tags t ON nt.tag_id = t.id
            WHERE n.user_id = 
            "#,
        );
        query_builder.push_bind(user_id_str);

        if let Some(pinned) = filter.is_pinned {
            query_builder
                .push(" AND n.is_pinned = ")
                .push_bind(if pinned { 1i32 } else { 0i32 });
        }

        if let Some(archived) = filter.is_archived {
            query_builder
                .push(" AND n.is_archived = ")
                .push_bind(if archived { 1i32 } else { 0i32 });
        }

        if let Some(tag_id) = filter.tag_id {
            query_builder
                .push(" AND n.id IN (SELECT note_id FROM note_tags WHERE tag_id = ")
                .push_bind(tag_id.to_string())
                .push(")");
        }

        query_builder.push(" GROUP BY n.id ORDER BY n.is_pinned DESC, n.updated_at DESC");

        let rows: Vec<NoteRowWithTags> = query_builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        rows.into_iter().map(|row| row.try_into_note()).collect()
    }

    async fn save(&self, note: &Note) -> DomainResult<()> {
        let id = note.id.to_string();
        let user_id = note.user_id.to_string();
        let is_pinned: i32 = if note.is_pinned { 1 } else { 0 };
        let is_archived: i32 = if note.is_archived { 1 } else { 0 };
        let created_at = note.created_at.to_rfc3339();
        let updated_at = note.updated_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO notes (id, user_id, title, content, color, is_pinned, is_archived, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                content = excluded.content,
                color = excluded.color,
                is_pinned = excluded.is_pinned,
                is_archived = excluded.is_archived,
                updated_at = excluded.updated_at
            "#
        )
        .bind(&id)
        .bind(&user_id)
        .bind(&note.title)
        .bind(&note.content)
        .bind(&note.color)
        .bind(is_pinned)
        .bind(is_archived)
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> DomainResult<()> {
        let id_str = id.to_string();
        sqlx::query("DELETE FROM notes WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn search(&self, user_id: Uuid, query: &str) -> DomainResult<Vec<Note>> {
        let user_id_str = user_id.to_string();
        let like_query = format!("%{}%", query);

        // Use FTS5 for full-text search OR tag name match, with JSON-aggregated tags
        let rows: Vec<NoteRowWithTags> = sqlx::query_as(
            r#"
            SELECT n.id, n.user_id, n.title, n.content, n.color, n.is_pinned, n.is_archived,
                   n.created_at, n.updated_at,
                   json_group_array(
                       CASE WHEN t.id IS NOT NULL
                       THEN json_object('id', t.id, 'name', t.name, 'user_id', t.user_id)
                       ELSE NULL END
                   ) as tags_json
            FROM notes n
            LEFT JOIN note_tags nt ON n.id = nt.note_id
            LEFT JOIN tags t ON nt.tag_id = t.id
            WHERE n.user_id = ? 
            AND (
                n.rowid IN (SELECT rowid FROM notes_fts WHERE notes_fts MATCH ?)
                OR
                EXISTS (
                    SELECT 1 FROM note_tags nt2 
                    JOIN tags t2 ON nt2.tag_id = t2.id 
                    WHERE nt2.note_id = n.id AND t2.name LIKE ?
                )
            )
            GROUP BY n.id
            ORDER BY n.updated_at DESC
            "#,
        )
        .bind(&user_id_str)
        .bind(query)
        .bind(like_query)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        rows.into_iter().map(|row| row.try_into_note()).collect()
    }

    async fn save_version(&self, version: &NoteVersion) -> DomainResult<()> {
        let id = version.id.to_string();
        let note_id = version.note_id.to_string();
        let created_at = version.created_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO note_versions (id, note_id, title, content, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&note_id)
        .bind(&version.title)
        .bind(&version.content)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        Ok(())
    }

    async fn find_versions_by_note_id(&self, note_id: Uuid) -> DomainResult<Vec<NoteVersion>> {
        let note_id_str = note_id.to_string();

        let rows: Vec<NoteVersionRow> = sqlx::query_as(
            r#"
            SELECT id, note_id, title, content, created_at
            FROM note_versions
            WHERE note_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(&note_id_str)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::RepositoryError(e.to_string()))?;

        let mut versions = Vec::with_capacity(rows.len());
        for row in rows {
            versions.push(row.try_into_version()?);
        }

        Ok(versions)
    }
}
