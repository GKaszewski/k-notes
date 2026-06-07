use async_trait::async_trait;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};

use domain::{
    errors::{DomainError, DomainResult},
    note::{
        entity::{Note, NoteFilter, NoteId, NoteVersion},
        ports::NoteRepository,
        value_objects::{NoteColor, NoteTitle},
    },
    tag::entity::{Tag, TagId},
    user::entity::UserId,
};

use crate::db::{RepoExt, parse_dt};

pub struct SqliteNoteRepository {
    pool: SqlitePool,
}

impl SqliteNoteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

// ── Row types ────────────────────────────────────────────────────────────────

#[derive(FromRow)]
struct NoteRow {
    id: String,
    user_id: String,
    title: Option<String>,
    content: String,
    color: String,
    is_pinned: i32,
    is_archived: i32,
    created_at: String,
    updated_at: String,
    tags_json: String,
}

impl TryFrom<NoteRow> for Note {
    type Error = DomainError;

    fn try_from(row: NoteRow) -> Result<Self, Self::Error> {
        let id = NoteId::from_uuid(
            uuid::Uuid::parse_str(&row.id)
                .map_err(|e| DomainError::Repository(format!("invalid note uuid: {e}")))?,
        );
        let user_id = UserId::from_uuid(
            uuid::Uuid::parse_str(&row.user_id)
                .map_err(|e| DomainError::Repository(format!("invalid user uuid: {e}")))?,
        );
        let title = NoteTitle::from_optional(row.title)?;
        let tags = parse_tags_json(&row.tags_json)?;

        Ok(Note {
            id,
            user_id,
            title,
            content: row.content,
            color: NoteColor::new(row.color),
            is_pinned: row.is_pinned != 0,
            is_archived: row.is_archived != 0,
            created_at: parse_dt(&row.created_at)?,
            updated_at: parse_dt(&row.updated_at)?,
            tags,
        })
    }
}

fn parse_tags_json(json: &str) -> Result<Vec<Tag>, DomainError> {
    let values: Vec<serde_json::Value> = serde_json::from_str(json)
        .map_err(|e| DomainError::Repository(format!("invalid tags json: {e}")))?;

    values
        .into_iter()
        .filter(|v| !v.is_null())
        .map(|v| {
            let parse_str = |key: &str| {
                v[key]
                    .as_str()
                    .ok_or_else(|| DomainError::Repository(format!("missing tag field '{key}'")))
            };
            let id = TagId::from_uuid(
                uuid::Uuid::parse_str(parse_str("id")?)
                    .map_err(|e| DomainError::Repository(format!("invalid tag uuid: {e}")))?,
            );
            let user_id = UserId::from_uuid(
                uuid::Uuid::parse_str(parse_str("user_id")?)
                    .map_err(|e| DomainError::Repository(format!("invalid tag user_id: {e}")))?,
            );
            let name = domain::tag::value_objects::TagName::new(parse_str("name")?)?;
            Ok(Tag::from_row(id, name, user_id))
        })
        .collect()
}

#[derive(FromRow)]
struct VersionRow {
    id: String,
    note_id: String,
    title: Option<String>,
    content: String,
    created_at: String,
}

impl TryFrom<VersionRow> for NoteVersion {
    type Error = DomainError;

    fn try_from(row: VersionRow) -> Result<Self, Self::Error> {
        Ok(NoteVersion {
            id: uuid::Uuid::parse_str(&row.id)
                .map_err(|e| DomainError::Repository(format!("invalid version uuid: {e}")))?,
            note_id: NoteId::from_uuid(
                uuid::Uuid::parse_str(&row.note_id)
                    .map_err(|e| DomainError::Repository(format!("invalid note uuid: {e}")))?,
            ),
            title: row.title,
            content: row.content,
            created_at: parse_dt(&row.created_at)?,
        })
    }
}

// ── Shared SELECT fragment ────────────────────────────────────────────────────

const NOTE_SELECT: &str = r#"
    SELECT n.id, n.user_id, n.title, n.content, n.color, n.is_pinned, n.is_archived,
           n.created_at, n.updated_at,
           json_group_array(
               CASE WHEN t.id IS NOT NULL
               THEN json_object('id', t.id, 'name', t.name, 'user_id', t.user_id)
               ELSE NULL END
           ) AS tags_json
    FROM notes n
    LEFT JOIN note_tags nt ON n.id = nt.note_id
    LEFT JOIN tags t ON nt.tag_id = t.id
"#;

// ── NoteRepository ───────────────────────────────────────────────────────────

#[async_trait]
impl NoteRepository for SqliteNoteRepository {
    async fn find_by_id(&self, id: &NoteId) -> DomainResult<Option<Note>> {
        let sql = format!("{NOTE_SELECT} WHERE n.id = ? GROUP BY n.id");
        sqlx::query_as::<_, NoteRow>(&sql)
            .bind(id.as_uuid().to_string())
            .fetch_optional(&self.pool)
            .await
            .repo()?
            .map(Note::try_from)
            .transpose()
    }

    async fn find_by_user(&self, user_id: &UserId, filter: NoteFilter) -> DomainResult<Vec<Note>> {
        let base = format!("{NOTE_SELECT} WHERE n.user_id = ");
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(base);
        qb.push_bind(user_id.as_uuid().to_string());

        if let Some(pinned) = filter.is_pinned {
            qb.push(" AND n.is_pinned = ").push_bind(pinned as i32);
        }
        if let Some(archived) = filter.is_archived {
            qb.push(" AND n.is_archived = ").push_bind(archived as i32);
        }
        if let Some(tag_id) = filter.tag_id {
            qb.push(" AND n.id IN (SELECT note_id FROM note_tags WHERE tag_id = ")
                .push_bind(tag_id.as_uuid().to_string())
                .push(")");
        }

        qb.push(" GROUP BY n.id ORDER BY n.is_pinned DESC, n.updated_at DESC");

        qb.build_query_as::<NoteRow>()
            .fetch_all(&self.pool)
            .await
            .repo()?
            .into_iter()
            .map(Note::try_from)
            .collect()
    }

    async fn search(&self, user_id: &UserId, query: &str) -> DomainResult<Vec<Note>> {
        let sql = format!(
            r#"{NOTE_SELECT}
            WHERE n.user_id = ?
            AND (
                n.rowid IN (SELECT rowid FROM notes_fts WHERE notes_fts MATCH ?)
                OR EXISTS (
                    SELECT 1 FROM note_tags nt2
                    JOIN tags t2 ON nt2.tag_id = t2.id
                    WHERE nt2.note_id = n.id AND t2.name LIKE ?
                )
            )
            GROUP BY n.id ORDER BY n.updated_at DESC"#
        );

        sqlx::query_as::<_, NoteRow>(&sql)
            .bind(user_id.as_uuid().to_string())
            .bind(query)
            .bind(format!("%{query}%"))
            .fetch_all(&self.pool)
            .await
            .repo()?
            .into_iter()
            .map(Note::try_from)
            .collect()
    }

    async fn save(&self, note: &Note) -> DomainResult<()> {
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
            "#,
        )
        .bind(note.id.as_uuid().to_string())
        .bind(note.user_id.as_uuid().to_string())
        .bind(note.title.as_ref().map(|t| t.as_ref()))
        .bind(&note.content)
        .bind(note.color.as_str())
        .bind(note.is_pinned as i32)
        .bind(note.is_archived as i32)
        .bind(note.created_at.to_rfc3339())
        .bind(note.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .repo()
        .map(|_| ())
    }

    async fn delete(&self, id: &NoteId) -> DomainResult<()> {
        sqlx::query("DELETE FROM notes WHERE id = ?")
            .bind(id.as_uuid().to_string())
            .execute(&self.pool)
            .await
            .repo()
            .map(|_| ())
    }

    async fn save_version(&self, version: &NoteVersion) -> DomainResult<()> {
        sqlx::query(
            "INSERT INTO note_versions (id, note_id, title, content, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(version.id.to_string())
        .bind(version.note_id.as_uuid().to_string())
        .bind(version.title.as_deref())
        .bind(&version.content)
        .bind(version.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .repo()
        .map(|_| ())
    }

    async fn find_versions(&self, note_id: &NoteId) -> DomainResult<Vec<NoteVersion>> {
        sqlx::query_as::<_, VersionRow>(
            "SELECT id, note_id, title, content, created_at FROM note_versions WHERE note_id = ? ORDER BY created_at DESC",
        )
        .bind(note_id.as_uuid().to_string())
        .fetch_all(&self.pool)
        .await
        .repo()?
        .into_iter()
        .map(NoteVersion::try_from)
        .collect()
    }
}

#[cfg(test)]
#[path = "tests/note.rs"]
mod tests;
