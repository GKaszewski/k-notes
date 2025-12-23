//! Domain services for K-Notes
//!
//! Services orchestrate business logic, enforce rules, and coordinate
//! between repositories. They are the "use cases" of the application.

use std::sync::Arc;
use uuid::Uuid;

use crate::entities::{MAX_TAGS_PER_NOTE, Note, NoteFilter, Tag, User};
use crate::errors::{DomainError, DomainResult};
use crate::repositories::{NoteRepository, TagRepository, UserRepository};

/// Request to create a new note
#[derive(Debug, Clone)]
pub struct CreateNoteRequest {
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
    pub color: Option<String>,
    pub is_pinned: bool,
}

/// Request to update an existing note
#[derive(Debug, Clone)]
pub struct UpdateNoteRequest {
    pub id: Uuid,
    pub user_id: Uuid, // For authorization check
    pub title: Option<String>,
    pub content: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
    pub color: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Service for Note operations
pub struct NoteService {
    note_repo: Arc<dyn NoteRepository>,
    tag_repo: Arc<dyn TagRepository>,
}

impl NoteService {
    pub fn new(note_repo: Arc<dyn NoteRepository>, tag_repo: Arc<dyn TagRepository>) -> Self {
        Self {
            note_repo,
            tag_repo,
        }
    }

    /// Create a new note with optional tags
    pub async fn create_note(&self, req: CreateNoteRequest) -> DomainResult<Note> {
        // Validate title is not empty
        if req.title.trim().is_empty() {
            return Err(DomainError::validation("Title cannot be empty"));
        }

        // Validate tag count
        if req.tags.len() > MAX_TAGS_PER_NOTE {
            return Err(DomainError::tag_limit_exceeded(req.tags.len()));
        }

        // Create the note
        let mut note = Note::new(req.user_id, req.title, req.content);
        note.is_pinned = req.is_pinned;
        if let Some(color) = req.color {
            note.set_color(color);
        }

        // Process tags
        for tag_name in &req.tags {
            let tag = self.get_or_create_tag(req.user_id, tag_name).await?;
            note.tags.push(tag);
        }

        // Save the note
        self.note_repo.save(&note).await?;

        // Associate tags with the note
        for tag in &note.tags {
            self.tag_repo.add_to_note(tag.id, note.id).await?;
        }

        Ok(note)
    }

    /// Update an existing note
    pub async fn update_note(&self, req: UpdateNoteRequest) -> DomainResult<Note> {
        // Find the note
        let mut note = self
            .note_repo
            .find_by_id(req.id)
            .await?
            .ok_or(DomainError::NoteNotFound(req.id))?;

        // Authorization check
        if note.user_id != req.user_id {
            return Err(DomainError::unauthorized(
                "Cannot modify another user's note",
            ));
        }

        // Apply updates
        if let Some(title) = req.title {
            if title.trim().is_empty() {
                return Err(DomainError::validation("Title cannot be empty"));
            }
            note.set_title(title);
        }

        if let Some(content) = req.content {
            note.set_content(content);
        }

        if let Some(pinned) = req.is_pinned {
            note.set_pinned(pinned);
        }

        if let Some(archived) = req.is_archived {
            note.set_archived(archived);
        }

        if let Some(color) = req.color {
            note.set_color(color);
        }

        // Handle tag updates
        if let Some(tag_names) = req.tags {
            if tag_names.len() > MAX_TAGS_PER_NOTE {
                return Err(DomainError::tag_limit_exceeded(tag_names.len()));
            }

            // Remove old tags
            for tag in &note.tags {
                self.tag_repo.remove_from_note(tag.id, note.id).await?;
            }

            // Add new tags
            note.tags.clear();
            for tag_name in &tag_names {
                let tag = self.get_or_create_tag(note.user_id, tag_name).await?;
                self.tag_repo.add_to_note(tag.id, note.id).await?;
                note.tags.push(tag);
            }
        }

        self.note_repo.save(&note).await?;
        Ok(note)
    }

    /// Get a note by ID with authorization check
    pub async fn get_note(&self, id: Uuid, user_id: Uuid) -> DomainResult<Note> {
        let note = self
            .note_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::NoteNotFound(id))?;

        if note.user_id != user_id {
            return Err(DomainError::unauthorized(
                "Cannot access another user's note",
            ));
        }

        Ok(note)
    }

    /// List notes for a user with optional filters
    pub async fn list_notes(&self, user_id: Uuid, filter: NoteFilter) -> DomainResult<Vec<Note>> {
        self.note_repo.find_by_user(user_id, filter).await
    }

    /// Delete a note with authorization check
    pub async fn delete_note(&self, id: Uuid, user_id: Uuid) -> DomainResult<()> {
        let note = self
            .note_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::NoteNotFound(id))?;

        if note.user_id != user_id {
            return Err(DomainError::unauthorized(
                "Cannot delete another user's note",
            ));
        }

        // Remove tag associations
        for tag in &note.tags {
            self.tag_repo.remove_from_note(tag.id, id).await?;
        }

        self.note_repo.delete(id).await
    }

    /// Search notes by query
    pub async fn search_notes(&self, user_id: Uuid, query: &str) -> DomainResult<Vec<Note>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        self.note_repo.search(user_id, query).await
    }

    /// Get or create a tag by name
    async fn get_or_create_tag(&self, user_id: Uuid, name: &str) -> DomainResult<Tag> {
        let name = name.trim().to_lowercase();
        if let Some(tag) = self.tag_repo.find_by_name(user_id, &name).await? {
            Ok(tag)
        } else {
            let tag = Tag::new(name, user_id);
            self.tag_repo.save(&tag).await?;
            Ok(tag)
        }
    }
}

/// Service for Tag operations
pub struct TagService {
    tag_repo: Arc<dyn TagRepository>,
}

impl TagService {
    pub fn new(tag_repo: Arc<dyn TagRepository>) -> Self {
        Self { tag_repo }
    }

    /// Create a new tag
    pub async fn create_tag(&self, user_id: Uuid, name: &str) -> DomainResult<Tag> {
        let name = name.trim().to_lowercase();
        if name.is_empty() {
            return Err(DomainError::validation("Tag name cannot be empty"));
        }

        // Check if tag already exists
        if self.tag_repo.find_by_name(user_id, &name).await?.is_some() {
            return Err(DomainError::TagAlreadyExists(name));
        }

        let tag = Tag::new(name, user_id);
        self.tag_repo.save(&tag).await?;
        Ok(tag)
    }

    /// List all tags for a user
    pub async fn list_tags(&self, user_id: Uuid) -> DomainResult<Vec<Tag>> {
        self.tag_repo.find_by_user(user_id).await
    }

    /// Delete a tag
    pub async fn delete_tag(&self, id: Uuid, user_id: Uuid) -> DomainResult<()> {
        let tag = self
            .tag_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::TagNotFound(id))?;

        if tag.user_id != user_id {
            return Err(DomainError::unauthorized(
                "Cannot delete another user's tag",
            ));
        }

        self.tag_repo.delete(id).await
    }
}

/// Service for User operations (OIDC-ready)
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    /// Find or create a user by OIDC subject
    /// This is the main entry point for OIDC authentication
    pub async fn find_or_create_by_subject(
        &self,
        subject: &str,
        email: &str,
    ) -> DomainResult<User> {
        if let Some(user) = self.user_repo.find_by_subject(subject).await? {
            Ok(user)
        } else {
            let user = User::new(subject, email);
            self.user_repo.save(&user).await?;
            Ok(user)
        }
    }

    /// Get a user by ID
    pub async fn get_user(&self, id: Uuid) -> DomainResult<User> {
        self.user_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::UserNotFound(id))
    }

    /// Delete a user and all associated data
    pub async fn delete_user(&self, id: Uuid) -> DomainResult<()> {
        // Note: In practice, we'd also need to delete notes and tags
        // This would be handled by cascade delete in the database
        // or by coordinating with other services
        self.user_repo.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::tests::MockNoteRepository;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock implementations for testing
    struct MockTagRepository {
        tags: Mutex<HashMap<Uuid, Tag>>,
        note_tags: Mutex<HashMap<(Uuid, Uuid), ()>>,
    }

    impl MockTagRepository {
        fn new() -> Self {
            Self {
                tags: Mutex::new(HashMap::new()),
                note_tags: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl TagRepository for MockTagRepository {
        async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Tag>> {
            Ok(self.tags.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_user(&self, user_id: Uuid) -> DomainResult<Vec<Tag>> {
            Ok(self
                .tags
                .lock()
                .unwrap()
                .values()
                .filter(|t| t.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn find_by_name(&self, user_id: Uuid, name: &str) -> DomainResult<Option<Tag>> {
            Ok(self
                .tags
                .lock()
                .unwrap()
                .values()
                .find(|t| t.user_id == user_id && t.name == name)
                .cloned())
        }

        async fn save(&self, tag: &Tag) -> DomainResult<()> {
            self.tags.lock().unwrap().insert(tag.id, tag.clone());
            Ok(())
        }

        async fn delete(&self, id: Uuid) -> DomainResult<()> {
            self.tags.lock().unwrap().remove(&id);
            Ok(())
        }

        async fn add_to_note(&self, tag_id: Uuid, note_id: Uuid) -> DomainResult<()> {
            self.note_tags.lock().unwrap().insert((tag_id, note_id), ());
            Ok(())
        }

        async fn remove_from_note(&self, tag_id: Uuid, note_id: Uuid) -> DomainResult<()> {
            self.note_tags.lock().unwrap().remove(&(tag_id, note_id));
            Ok(())
        }

        async fn find_by_note(&self, note_id: Uuid) -> DomainResult<Vec<Tag>> {
            let note_tags = self.note_tags.lock().unwrap();
            let tags = self.tags.lock().unwrap();
            Ok(note_tags
                .keys()
                .filter(|(_, nid)| *nid == note_id)
                .filter_map(|(tid, _)| tags.get(tid).cloned())
                .collect())
        }
    }

    struct MockUserRepository {
        users: Mutex<HashMap<Uuid, User>>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<User>> {
            Ok(self.users.lock().unwrap().get(&id).cloned())
        }

        async fn find_by_subject(&self, subject: &str) -> DomainResult<Option<User>> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .values()
                .find(|u| u.subject == subject)
                .cloned())
        }

        async fn find_by_email(&self, email: &str) -> DomainResult<Option<User>> {
            Ok(self
                .users
                .lock()
                .unwrap()
                .values()
                .find(|u| u.email == email)
                .cloned())
        }

        async fn save(&self, user: &User) -> DomainResult<()> {
            self.users.lock().unwrap().insert(user.id, user.clone());
            Ok(())
        }

        async fn delete(&self, id: Uuid) -> DomainResult<()> {
            self.users.lock().unwrap().remove(&id);
            Ok(())
        }
    }

    mod note_service_tests {
        use super::*;

        fn create_note_service() -> (NoteService, Uuid) {
            let note_repo = Arc::new(MockNoteRepository::new());
            let tag_repo = Arc::new(MockTagRepository::new());
            let user_id = Uuid::new_v4();
            (NoteService::new(note_repo, tag_repo), user_id)
        }

        #[tokio::test]
        async fn test_create_note_success() {
            let (service, user_id) = create_note_service();

            let req = CreateNoteRequest {
                user_id,
                title: "My Note".to_string(),
                content: "# Hello World".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };

            let note = service.create_note(req).await.unwrap();

            assert_eq!(note.title, "My Note");
            assert_eq!(note.content, "# Hello World");
            assert_eq!(note.user_id, user_id);
            assert_eq!(note.color, "DEFAULT");
            assert!(!note.is_pinned);
        }

        #[tokio::test]
        async fn test_create_note_with_tags() {
            let (service, user_id) = create_note_service();

            let req = CreateNoteRequest {
                user_id,
                title: "Tagged Note".to_string(),
                content: "Content".to_string(),
                tags: vec!["work".to_string(), "important".to_string()],
                color: None,
                is_pinned: false,
            };

            let note = service.create_note(req).await.unwrap();

            assert_eq!(note.tags.len(), 2);
            assert!(note.tags.iter().any(|t| t.name == "work"));
            assert!(note.tags.iter().any(|t| t.name == "important"));
        }

        #[tokio::test]
        async fn test_create_note_empty_title_fails() {
            let (service, user_id) = create_note_service();

            let req = CreateNoteRequest {
                user_id,
                title: "   ".to_string(), // Whitespace only
                content: "Content".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };

            let result = service.create_note(req).await;
            assert!(matches!(result, Err(DomainError::ValidationError(_))));
        }

        #[tokio::test]
        async fn test_create_note_too_many_tags_fails() {
            let (service, user_id) = create_note_service();

            let tags: Vec<String> = (0..=MAX_TAGS_PER_NOTE)
                .map(|i| format!("tag-{}", i))
                .collect();

            let req = CreateNoteRequest {
                user_id,
                title: "Note".to_string(),
                content: "Content".to_string(),
                tags,
                color: None,
                is_pinned: false,
            };

            let result = service.create_note(req).await;
            assert!(matches!(result, Err(DomainError::TagLimitExceeded { .. })));
        }

        #[tokio::test]
        async fn test_update_note_success() {
            let (service, user_id) = create_note_service();

            // Create a note first
            let create_req = CreateNoteRequest {
                user_id,
                title: "Original".to_string(),
                content: "Original content".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };
            let note = service.create_note(create_req).await.unwrap();

            // Update it
            let update_req = UpdateNoteRequest {
                id: note.id,
                user_id,
                title: Some("Updated".to_string()),
                content: None,
                is_pinned: Some(true),
                is_archived: None,
                color: Some("red".to_string()),
                tags: None,
            };
            let updated = service.update_note(update_req).await.unwrap();

            assert_eq!(updated.title, "Updated");
            assert_eq!(updated.content, "Original content"); // Unchanged
            assert!(updated.is_pinned);
            assert_eq!(updated.color, "red");
        }

        #[tokio::test]
        async fn test_update_note_unauthorized() {
            let (service, user_id) = create_note_service();
            let other_user = Uuid::new_v4();

            // Create a note
            let create_req = CreateNoteRequest {
                user_id,
                title: "My Note".to_string(),
                content: "Content".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };
            let note = service.create_note(create_req).await.unwrap();

            // Try to update with different user
            let update_req = UpdateNoteRequest {
                id: note.id,
                user_id: other_user,
                title: Some("Hacked".to_string()),
                content: None,
                is_pinned: None,
                is_archived: None,
                color: None,
                tags: None,
            };
            let result = service.update_note(update_req).await;

            assert!(matches!(result, Err(DomainError::Unauthorized(_))));
        }

        #[tokio::test]
        async fn test_delete_note_success() {
            let (service, user_id) = create_note_service();

            let create_req = CreateNoteRequest {
                user_id,
                title: "To Delete".to_string(),
                content: "Content".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };
            let note = service.create_note(create_req).await.unwrap();

            service.delete_note(note.id, user_id).await.unwrap();

            let result = service.get_note(note.id, user_id).await;
            assert!(matches!(result, Err(DomainError::NoteNotFound(_))));
        }

        #[tokio::test]
        async fn test_search_empty_query_returns_empty() {
            let (service, user_id) = create_note_service();

            let results = service.search_notes(user_id, "   ").await.unwrap();
            assert!(results.is_empty());
        }
    }

    mod tag_service_tests {
        use super::*;

        fn create_tag_service() -> (TagService, Uuid) {
            let tag_repo = Arc::new(MockTagRepository::new());
            let user_id = Uuid::new_v4();
            (TagService::new(tag_repo), user_id)
        }

        #[tokio::test]
        async fn test_create_tag_success() {
            let (service, user_id) = create_tag_service();

            let tag = service.create_tag(user_id, "Work").await.unwrap();

            assert_eq!(tag.name, "work"); // Lowercase
            assert_eq!(tag.user_id, user_id);
        }

        #[tokio::test]
        async fn test_create_tag_empty_fails() {
            let (service, user_id) = create_tag_service();

            let result = service.create_tag(user_id, "   ").await;
            assert!(matches!(result, Err(DomainError::ValidationError(_))));
        }

        #[tokio::test]
        async fn test_create_duplicate_tag_fails() {
            let (service, user_id) = create_tag_service();

            service.create_tag(user_id, "work").await.unwrap();
            let result = service.create_tag(user_id, "WORK").await; // Case-insensitive

            assert!(matches!(result, Err(DomainError::TagAlreadyExists(_))));
        }
    }

    mod user_service_tests {
        use super::*;

        fn create_user_service() -> UserService {
            let user_repo = Arc::new(MockUserRepository::new());
            UserService::new(user_repo)
        }

        #[tokio::test]
        async fn test_find_or_create_creates_new_user() {
            let service = create_user_service();

            let user = service
                .find_or_create_by_subject("oidc|123", "test@example.com")
                .await
                .unwrap();

            assert_eq!(user.subject, "oidc|123");
            assert_eq!(user.email, "test@example.com");
        }

        #[tokio::test]
        async fn test_find_or_create_returns_existing_user() {
            let service = create_user_service();

            let user1 = service
                .find_or_create_by_subject("oidc|123", "test@example.com")
                .await
                .unwrap();

            let user2 = service
                .find_or_create_by_subject("oidc|123", "test@example.com")
                .await
                .unwrap();

            assert_eq!(user1.id, user2.id);
        }
    }
}
