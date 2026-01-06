//! Domain services for K-Notes
//!
//! Services orchestrate business logic, enforce rules, and coordinate
//! between repositories. They are the \"use cases\" of the application.

use std::sync::Arc;
use uuid::Uuid;

use crate::entities::{MAX_TAGS_PER_NOTE, Note, NoteFilter, NoteVersion, Tag, User};
use crate::errors::{DomainError, DomainResult};
use crate::ports::MessageBroker;
use crate::repositories::{NoteRepository, TagRepository, UserRepository};
use crate::value_objects::{Email, NoteTitle, TagName};

/// Request to create a new note
#[derive(Debug, Clone)]
pub struct CreateNoteRequest {
    pub user_id: Uuid,
    /// Title is optional - notes can have no title
    pub title: Option<NoteTitle>,
    pub content: String,
    /// Tags are pre-validated TagName values
    pub tags: Vec<TagName>,
    pub color: Option<String>,
    pub is_pinned: bool,
}

/// Request to update an existing note
#[derive(Debug, Clone)]
pub struct UpdateNoteRequest {
    pub id: Uuid,
    pub user_id: Uuid, // For authorization check
    /// None means "don't change", Some(None) means "remove title", Some(Some(t)) means "set title"
    pub title: Option<Option<NoteTitle>>,
    pub content: Option<String>,
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
    pub color: Option<String>,
    /// Pre-validated TagName values
    pub tags: Option<Vec<TagName>>,
}

/// Service for Note operations
pub struct NoteService {
    note_repo: Arc<dyn NoteRepository>,
    tag_repo: Arc<dyn TagRepository>,
    message_broker: Option<Arc<dyn MessageBroker>>,
}

impl NoteService {
    pub fn new(note_repo: Arc<dyn NoteRepository>, tag_repo: Arc<dyn TagRepository>) -> Self {
        Self {
            note_repo,
            tag_repo,
            message_broker: None,
        }
    }

    /// Builder method to set the message broker
    pub fn with_message_broker(mut self, broker: Arc<dyn MessageBroker>) -> Self {
        self.message_broker = Some(broker);
        self
    }

    /// Helper to publish note update events
    async fn publish_note_event(&self, note: &Note) {
        if let Some(ref broker) = self.message_broker {
            if let Err(e) = broker.publish_note_updated(note).await {
                tracing::error!(note_id = %note.id, "Failed to publish note event: {}", e);
            } else {
                tracing::info!(note_id = %note.id, "Published note.updated event");
            }
        }
    }

    /// Create a new note with optional tags
    pub async fn create_note(&self, req: CreateNoteRequest) -> DomainResult<Note> {
        // Title validation is handled by NoteTitle type - no need for runtime check
        // Tags are pre-validated as TagName values

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
            let tag = self
                .get_or_create_tag(req.user_id, tag_name.clone())
                .await?;
            note.tags.push(tag);
        }

        // Save the note
        self.note_repo.save(&note).await?;

        // Associate tags with the note
        for tag in &note.tags {
            self.tag_repo.add_to_note(tag.id, note.id).await?;
        }

        // Publish event for smart features processing
        self.publish_note_event(&note).await;

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

        // Create version snapshot (save current state)
        let version = NoteVersion::new(
            note.id,
            note.title.as_ref().map(|t| t.as_ref().to_string()),
            note.content.clone(),
        );
        self.note_repo.save_version(&version).await?;

        // Apply updates - title is already validated via NoteTitle type
        if let Some(title) = req.title {
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
            for tag_name in tag_names {
                let tag = self.get_or_create_tag(note.user_id, tag_name).await?;
                self.tag_repo.add_to_note(tag.id, note.id).await?;
                note.tags.push(tag);
            }
        }

        self.note_repo.save(&note).await?;

        // Publish event for smart features processing
        self.publish_note_event(&note).await;

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

    /// List versions of a note
    pub async fn list_note_versions(
        &self,
        note_id: Uuid,
        user_id: Uuid,
    ) -> DomainResult<Vec<crate::entities::NoteVersion>> {
        // Verify access (re-using get_note for authorization check)
        self.get_note(note_id, user_id).await?;

        self.note_repo.find_versions_by_note_id(note_id).await
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
    ///
    /// Handles race conditions gracefully: if a concurrent request creates
    /// the same tag, we catch the unique constraint violation and retry the lookup.
    async fn get_or_create_tag(&self, user_id: Uuid, name: TagName) -> DomainResult<Tag> {
        // First, try to find existing tag
        if let Some(tag) = self.tag_repo.find_by_name(user_id, name.as_ref()).await? {
            return Ok(tag);
        }

        // Tag doesn't exist, try to create it
        let tag = Tag::new(name.clone(), user_id);
        match self.tag_repo.save(&tag).await {
            Ok(()) => Ok(tag),
            Err(DomainError::RepositoryError(ref e)) if e.contains("UNIQUE constraint") => {
                // Race condition: another request created the tag between our check and save
                // Retry the lookup
                tracing::debug!(tag_name = %name, "Tag creation race condition detected, retrying lookup");
                self.tag_repo
                    .find_by_name(user_id, name.as_ref())
                    .await?
                    .ok_or_else(|| DomainError::validation("Tag creation race condition"))
            }
            Err(e) => Err(e),
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

    /// Create a new tag (TagName is pre-validated)
    pub async fn create_tag(&self, user_id: Uuid, name: TagName) -> DomainResult<Tag> {
        // Check if tag already exists
        if self
            .tag_repo
            .find_by_name(user_id, name.as_ref())
            .await?
            .is_some()
        {
            return Err(DomainError::TagAlreadyExists(name.into_inner()));
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

    /// Rename a tag (new_name is pre-validated TagName)
    pub async fn rename_tag(
        &self,
        id: Uuid,
        user_id: Uuid,
        new_name: TagName,
    ) -> DomainResult<Tag> {
        // Find the existing tag
        let mut tag = self
            .tag_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::TagNotFound(id))?;

        // Authorization check
        if tag.user_id != user_id {
            return Err(DomainError::unauthorized(
                "Cannot rename another user's tag",
            ));
        }

        // Check if new name already exists (and it's not the same tag)
        if let Some(existing) = self
            .tag_repo
            .find_by_name(user_id, new_name.as_ref())
            .await?
        {
            if existing.id != id {
                return Err(DomainError::TagAlreadyExists(new_name.into_inner()));
            }
        }

        // Update the name
        tag.name = new_name;
        self.tag_repo.save(&tag).await?;
        Ok(tag)
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

    pub async fn find_or_create(&self, subject: &str, email: &str) -> DomainResult<User> {
        // 1. Try to find by subject (OIDC id)
        if let Some(user) = self.user_repo.find_by_subject(subject).await? {
            return Ok(user);
        }

        // 2. Try to find by email
        if let Some(mut user) = self.user_repo.find_by_email(email).await? {
            // Link subject if missing (account linking logic)
            if user.subject != subject {
                user.subject = subject.to_string();
                self.user_repo.save(&user).await?;
            }
            return Ok(user);
        }

        // 3. Create new user
        let email = Email::try_from(email)?;
        let user = User::new(subject, email);
        self.user_repo.save(&user).await?;

        Ok(user)
    }

    pub async fn find_by_id(&self, id: Uuid) -> DomainResult<User> {
        self.user_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::UserNotFound(id))
    }

    pub async fn find_by_email(&self, email: &str) -> DomainResult<Option<User>> {
        self.user_repo.find_by_email(email).await
    }

    pub async fn create_local(&self, email: &str, password_hash: &str) -> DomainResult<User> {
        let email = Email::try_from(email)?;
        let user = User::new_local(email, password_hash);
        self.user_repo.save(&user).await?;
        Ok(user)
    }
}

/// Service for Smart Features (Embeddings, Vector Search, Linking)
pub struct SmartNoteService {
    embedding_generator: Arc<dyn crate::ports::EmbeddingGenerator>,
    vector_store: Arc<dyn crate::ports::VectorStore>,
    link_repo: Arc<dyn crate::ports::LinkRepository>,
}

impl SmartNoteService {
    pub fn new(
        embedding_generator: Arc<dyn crate::ports::EmbeddingGenerator>,
        vector_store: Arc<dyn crate::ports::VectorStore>,
        link_repo: Arc<dyn crate::ports::LinkRepository>,
    ) -> Self {
        Self {
            embedding_generator,
            vector_store,
            link_repo,
        }
    }

    /// Process a note to generate embeddings and find similar notes
    pub async fn process_note(&self, note: &Note) -> DomainResult<()> {
        // 1. Generate embedding
        let embedding = self
            .embedding_generator
            .generate_embedding(&note.content)
            .await?;

        // 2. Upsert to vector store
        self.vector_store.upsert(note.id, &embedding).await?;

        // 3. Find similar notes
        // TODO: Make limit configurable
        let similar = self.vector_store.find_similar(&embedding, 5).await?;

        // 4. Create links
        let links: Vec<crate::entities::NoteLink> = similar
            .into_iter()
            .filter(|(id, _)| *id != note.id) // Exclude self
            .map(|(target_id, score)| crate::entities::NoteLink::new(note.id, target_id, score))
            .collect();

        // 5. Save links (replacing old ones)
        if !links.is_empty() {
            self.link_repo.delete_links_for_source(note.id).await?;
            self.link_repo.save_links(&links).await?;
        }

        Ok(())
    }

    /// Get related notes for a given note ID
    pub async fn get_related_notes(
        &self,
        note_id: Uuid,
    ) -> DomainResult<Vec<crate::entities::NoteLink>> {
        self.link_repo.get_links_for_note(note_id).await
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
                .find(|t| t.user_id == user_id && t.name.as_ref() == name)
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
                .find(|u| u.email_str() == email)
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

            let title = NoteTitle::try_from("My Note").ok();
            let req = CreateNoteRequest {
                user_id,
                title,
                content: "# Hello World".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };

            let note = service.create_note(req).await.unwrap();

            assert_eq!(note.title_str(), "My Note");
            assert_eq!(note.content, "# Hello World");
            assert_eq!(note.user_id, user_id);
            assert_eq!(note.color, "DEFAULT");
            assert!(!note.is_pinned);
        }

        #[tokio::test]
        async fn test_create_note_without_title() {
            let (service, user_id) = create_note_service();

            let req = CreateNoteRequest {
                user_id,
                title: None,
                content: "Content without title".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };

            let note = service.create_note(req).await.unwrap();

            assert!(note.title.is_none());
            assert_eq!(note.title_str(), "");
            assert_eq!(note.content, "Content without title");
        }

        #[tokio::test]
        async fn test_create_note_with_tags() {
            let (service, user_id) = create_note_service();

            let title = NoteTitle::try_from("Tagged Note").ok();
            let tags = vec![
                TagName::try_from("work").unwrap(),
                TagName::try_from("important").unwrap(),
            ];
            let req = CreateNoteRequest {
                user_id,
                title,
                content: "Content".to_string(),
                tags,
                color: None,
                is_pinned: false,
            };

            let note = service.create_note(req).await.unwrap();

            assert_eq!(note.tags.len(), 2);
            assert!(note.tags.iter().any(|t| t.name_str() == "work"));
            assert!(note.tags.iter().any(|t| t.name_str() == "important"));
        }

        #[tokio::test]
        async fn test_create_note_too_many_tags_fails() {
            let (service, user_id) = create_note_service();

            let tags: Vec<TagName> = (0..=MAX_TAGS_PER_NOTE)
                .map(|i| TagName::try_from(format!("tag-{}", i)).unwrap())
                .collect();

            let title = NoteTitle::try_from("Note").ok();
            let req = CreateNoteRequest {
                user_id,
                title,
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
            let title = NoteTitle::try_from("Original").ok();
            let create_req = CreateNoteRequest {
                user_id,
                title,
                content: "Original content".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };
            let note = service.create_note(create_req).await.unwrap();

            // Update it
            let new_title = NoteTitle::try_from("Updated").ok();
            let update_req = UpdateNoteRequest {
                id: note.id,
                user_id,
                title: Some(new_title),
                content: None,
                is_pinned: Some(true),
                is_archived: None,
                color: Some("red".to_string()),
                tags: None,
            };
            let updated = service.update_note(update_req).await.unwrap();

            assert_eq!(updated.title_str(), "Updated");
            assert_eq!(updated.content, "Original content"); // Unchanged
            assert!(updated.is_pinned);
            assert_eq!(updated.color, "red");
        }

        #[tokio::test]
        async fn test_update_note_unauthorized() {
            let (service, user_id) = create_note_service();
            let other_user = Uuid::new_v4();

            // Create a note
            let title = NoteTitle::try_from("My Note").ok();
            let create_req = CreateNoteRequest {
                user_id,
                title,
                content: "Content".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };
            let note = service.create_note(create_req).await.unwrap();

            // Try to update with different user
            let new_title = NoteTitle::try_from("Hacked").ok();
            let update_req = UpdateNoteRequest {
                id: note.id,
                user_id: other_user,
                title: Some(new_title),
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

            let title = NoteTitle::try_from("To Delete").ok();
            let create_req = CreateNoteRequest {
                user_id,
                title,
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

        #[tokio::test]
        async fn test_update_note_creates_version() {
            let (service, user_id) = create_note_service();

            // Create original note
            let title = NoteTitle::try_from("Original Title").ok();
            let create_req = CreateNoteRequest {
                user_id,
                title,
                content: "Original Content".to_string(),
                tags: vec![],
                color: None,
                is_pinned: false,
            };
            let note = service.create_note(create_req).await.unwrap();

            // Update the note
            let new_title = NoteTitle::try_from("New Title").ok();
            let update_req = UpdateNoteRequest {
                id: note.id,
                user_id,
                title: Some(new_title),
                content: Some("New Content".to_string()),
                is_pinned: None,
                is_archived: None,
                color: None,
                tags: None,
            };
            service.update_note(update_req).await.unwrap();

            // Check if version was saved
            let versions = service
                .note_repo
                .find_versions_by_note_id(note.id)
                .await
                .unwrap();

            assert_eq!(versions.len(), 1);
            let version = &versions[0];
            assert_eq!(version.title, Some("Original Title".to_string()));
            assert_eq!(version.content, "Original Content");
            assert_eq!(version.note_id, note.id);
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

            let name = TagName::try_from("Work").unwrap();
            let tag = service.create_tag(user_id, name).await.unwrap();

            assert_eq!(tag.name_str(), "work"); // Lowercase
            assert_eq!(tag.user_id, user_id);
        }

        #[tokio::test]
        async fn test_create_duplicate_tag_fails() {
            let (service, user_id) = create_tag_service();

            let name1 = TagName::try_from("work").unwrap();
            service.create_tag(user_id, name1).await.unwrap();

            let name2 = TagName::try_from("WORK").unwrap(); // Case-insensitive
            let result = service.create_tag(user_id, name2).await;

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

            let email = Email::try_from("test@example.com").unwrap();
            let user = service
                .find_or_create("oidc|123", email.as_ref())
                .await
                .unwrap();

            assert_eq!(user.subject, "oidc|123");
            assert_eq!(user.email_str(), "test@example.com");
        }

        #[tokio::test]
        async fn test_find_or_create_returns_existing_user() {
            let service = create_user_service();

            let email1 = Email::try_from("test@example.com").unwrap();
            let user1 = service
                .find_or_create("oidc|123", email1.as_ref())
                .await
                .unwrap();

            let email2 = Email::try_from("test@example.com").unwrap();
            let user2 = service
                .find_or_create("oidc|123", email2.as_ref())
                .await
                .unwrap();

            assert_eq!(user1.id, user2.id);
        }
    }
}
