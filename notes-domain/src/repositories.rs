//! Repository ports (traits) for K-Notes
//!
//! These traits define the interface for data persistence without
//! specifying the implementation. This is the "port" in hexagonal architecture.
//! Concrete implementations (adapters) live in the `notes-infra` crate.

use async_trait::async_trait;
use uuid::Uuid;

use crate::entities::{Note, NoteFilter, Tag, User};
use crate::errors::DomainResult;

/// Repository port for Note persistence
#[async_trait]
pub trait NoteRepository: Send + Sync {
    /// Find a note by its ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Note>>;

    /// Find all notes for a user, optionally filtered
    async fn find_by_user(&self, user_id: Uuid, filter: NoteFilter) -> DomainResult<Vec<Note>>;

    /// Save a new note or update an existing one
    async fn save(&self, note: &Note) -> DomainResult<()>;

    /// Delete a note by its ID
    async fn delete(&self, id: Uuid) -> DomainResult<()>;

    /// Full-text search across note titles and content
    async fn search(&self, user_id: Uuid, query: &str) -> DomainResult<Vec<Note>>;

    /// Save a note version
    async fn save_version(&self, version: &crate::entities::NoteVersion) -> DomainResult<()>;

    /// Find all versions for a note
    async fn find_versions_by_note_id(
        &self,
        note_id: Uuid,
    ) -> DomainResult<Vec<crate::entities::NoteVersion>>;
}

/// Repository port for User persistence
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their internal ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<User>>;

    /// Find a user by their OIDC subject (used for authentication)
    async fn find_by_subject(&self, subject: &str) -> DomainResult<Option<User>>;

    /// Find a user by their email
    async fn find_by_email(&self, email: &str) -> DomainResult<Option<User>>;

    /// Save a new user or update an existing one
    async fn save(&self, user: &User) -> DomainResult<()>;

    /// Delete a user by their ID
    async fn delete(&self, id: Uuid) -> DomainResult<()>;
}

/// Repository port for Tag persistence
#[async_trait]
pub trait TagRepository: Send + Sync {
    /// Find a tag by its ID
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Tag>>;

    /// Find all tags for a user
    async fn find_by_user(&self, user_id: Uuid) -> DomainResult<Vec<Tag>>;

    /// Find a tag by name for a specific user
    async fn find_by_name(&self, user_id: Uuid, name: &str) -> DomainResult<Option<Tag>>;

    /// Save a new tag or update an existing one
    async fn save(&self, tag: &Tag) -> DomainResult<()>;

    /// Delete a tag by its ID
    async fn delete(&self, id: Uuid) -> DomainResult<()>;

    /// Add a tag to a note
    async fn add_to_note(&self, tag_id: Uuid, note_id: Uuid) -> DomainResult<()>;

    /// Remove a tag from a note
    async fn remove_from_note(&self, tag_id: Uuid, note_id: Uuid) -> DomainResult<()>;

    /// Get all tags for a specific note
    async fn find_by_note(&self, note_id: Uuid) -> DomainResult<Vec<Tag>>;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// In-memory mock implementation for testing
    pub struct MockNoteRepository {
        notes: Mutex<HashMap<Uuid, Note>>,
        versions: Mutex<HashMap<Uuid, Vec<crate::entities::NoteVersion>>>,
    }

    impl MockNoteRepository {
        pub fn new() -> Self {
            Self {
                notes: Mutex::new(HashMap::new()),
                versions: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl NoteRepository for MockNoteRepository {
        async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Note>> {
            Ok(self.notes.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_user(&self, user_id: Uuid, filter: NoteFilter) -> DomainResult<Vec<Note>> {
            let notes = self.notes.lock().unwrap();
            let mut result: Vec<Note> = notes
                .values()
                .filter(|n| n.user_id == user_id)
                .filter(|n| filter.is_pinned.is_none() || filter.is_pinned == Some(n.is_pinned))
                .filter(|n| {
                    filter.is_archived.is_none() || filter.is_archived == Some(n.is_archived)
                })
                .cloned()
                .collect();
            result.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            Ok(result)
        }

        async fn save(&self, note: &Note) -> DomainResult<()> {
            self.notes.lock().unwrap().insert(note.id, note.clone());
            Ok(())
        }

        async fn delete(&self, id: Uuid) -> DomainResult<()> {
            self.notes.lock().unwrap().remove(&id);
            Ok(())
        }

        async fn search(&self, user_id: Uuid, query: &str) -> DomainResult<Vec<Note>> {
            let notes = self.notes.lock().unwrap();
            let query_lower = query.to_lowercase();
            Ok(notes
                .values()
                .filter(|n| n.user_id == user_id)
                .filter(|n| {
                    n.title.to_lowercase().contains(&query_lower)
                        || n.content.to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect())
        }

        async fn save_version(&self, version: &crate::entities::NoteVersion) -> DomainResult<()> {
            let mut versions = self.versions.lock().unwrap();
            let note_versions = versions.entry(version.note_id).or_insert_with(Vec::new);
            note_versions.push(version.clone());
            Ok(())
        }

        async fn find_versions_by_note_id(
            &self,
            note_id: Uuid,
        ) -> DomainResult<Vec<crate::entities::NoteVersion>> {
            let versions = self.versions.lock().unwrap();
            Ok(versions.get(&note_id).cloned().unwrap_or_default())
        }
    }

    #[tokio::test]
    async fn test_mock_note_repository_save_and_find() {
        let repo = MockNoteRepository::new();
        let user_id = Uuid::new_v4();
        let note = Note::new(user_id, "Test Note", "Test content");
        let note_id = note.id;

        repo.save(&note).await.unwrap();
        let found = repo.find_by_id(note_id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Test Note");
    }

    #[tokio::test]
    async fn test_mock_note_repository_filter() {
        let repo = MockNoteRepository::new();
        let user_id = Uuid::new_v4();

        let mut pinned_note = Note::new(user_id, "Pinned", "Content");
        pinned_note.is_pinned = true;
        repo.save(&pinned_note).await.unwrap();

        let regular_note = Note::new(user_id, "Regular", "Content");
        repo.save(&regular_note).await.unwrap();

        let pinned_only = repo
            .find_by_user(user_id, NoteFilter::new().pinned())
            .await
            .unwrap();

        assert_eq!(pinned_only.len(), 1);
        assert_eq!(pinned_only[0].title, "Pinned");
    }

    #[tokio::test]
    async fn test_mock_note_repository_search() {
        let repo = MockNoteRepository::new();
        let user_id = Uuid::new_v4();

        let note1 = Note::new(user_id, "Shopping List", "Buy milk and eggs");
        let note2 = Note::new(user_id, "Meeting Notes", "Discuss project timeline");
        repo.save(&note1).await.unwrap();
        repo.save(&note2).await.unwrap();

        let results = repo.search(user_id, "milk").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Shopping List");

        let results = repo.search(user_id, "notes").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Meeting Notes");
    }
}
