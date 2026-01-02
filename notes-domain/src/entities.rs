//! Domain entities for K-Notes
//!
//! This module contains pure domain types with no I/O dependencies.
//! These represent the core business concepts of the application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::value_objects::{Email, NoteTitle, TagName};

/// Maximum number of tags allowed per note (business rule)
pub const MAX_TAGS_PER_NOTE: usize = 10;

/// A user in the system.
///
/// Designed to be OIDC-ready: the `subject` field stores the OIDC subject claim
/// for federated identity, while `email` is used for display purposes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    /// OIDC subject identifier (unique per identity provider)
    /// For local auth, this can be the same as email
    pub subject: String,
    /// Validated email address
    pub email: Email,
    /// Password hash for local authentication (Argon2 etc.)
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with the current timestamp
    pub fn new(subject: impl Into<String>, email: Email) -> Self {
        Self {
            id: Uuid::new_v4(),
            subject: subject.into(),
            email,
            password_hash: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new user with password hash
    pub fn new_local(email: Email, password_hash: impl Into<String>) -> Self {
        let subject = email.as_ref().to_string();
        Self {
            id: Uuid::new_v4(),
            subject, // Use email as subject for local auth
            email,
            password_hash: Some(password_hash.into()),
            created_at: Utc::now(),
        }
    }

    /// Create a user with a specific ID (for reconstruction from storage)
    /// This accepts raw strings for compatibility with database reads.
    pub fn with_id(
        id: Uuid,
        subject: impl Into<String>,
        email: Email,
        password_hash: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            subject: subject.into(),
            email,
            password_hash,
            created_at,
        }
    }

    /// Get email as string reference (convenience method)
    pub fn email_str(&self) -> &str {
        self.email.as_ref()
    }
}

/// A tag that can be attached to notes.
///
/// Tags are user-scoped, meaning each user has their own set of tags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    /// Validated tag name (1-50 chars, trimmed, lowercase)
    pub name: TagName,
    pub user_id: Uuid,
}

impl Tag {
    /// Create a new tag for a user
    pub fn new(name: TagName, user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            user_id,
        }
    }

    /// Create a tag with a specific ID (for reconstruction from storage)
    pub fn with_id(id: Uuid, name: TagName, user_id: Uuid) -> Self {
        Self { id, name, user_id }
    }

    /// Get name as string reference (convenience method)
    pub fn name_str(&self) -> &str {
        self.name.as_ref()
    }
}

/// A note containing user content.
///
/// Notes support Markdown content and can be pinned or archived.
/// Each note can have up to [`MAX_TAGS_PER_NOTE`] tags.
/// Title is optional - users may create notes without a title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub user_id: Uuid,
    /// Optional title (max 200 chars when present)
    pub title: Option<NoteTitle>,
    /// Content stored as Markdown text
    pub content: String,
    /// Background color of the note (hex or name)
    #[serde(default = "default_color")]
    pub color: String,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
}

fn default_color() -> String {
    "DEFAULT".to_string()
}

impl Note {
    /// Create a new note with the current timestamp
    pub fn new(user_id: Uuid, title: Option<NoteTitle>, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            title,
            content: content.into(),
            color: default_color(),
            is_pinned: false,
            is_archived: false,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }

    /// Set the color of the note
    pub fn set_color(&mut self, color: impl Into<String>) {
        self.color = color.into();
        self.updated_at = Utc::now();
    }

    /// Pin or unpin the note
    pub fn set_pinned(&mut self, pinned: bool) {
        self.is_pinned = pinned;
        self.updated_at = Utc::now();
    }

    /// Archive or unarchive the note
    pub fn set_archived(&mut self, archived: bool) {
        self.is_archived = archived;
        self.updated_at = Utc::now();
    }

    /// Update the note's title
    pub fn set_title(&mut self, title: Option<NoteTitle>) {
        self.title = title;
        self.updated_at = Utc::now();
    }

    /// Update the note's content
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.updated_at = Utc::now();
    }

    /// Check if adding a tag would exceed the limit
    pub fn can_add_tag(&self) -> bool {
        self.tags.len() < MAX_TAGS_PER_NOTE
    }

    /// Get the number of tags on this note
    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }

    /// Get title as string reference, returns empty string if None
    pub fn title_str(&self) -> &str {
        self.title.as_ref().map(|t| t.as_ref()).unwrap_or("")
    }
}

/// A snapshot of a note's state at a specific point in time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteVersion {
    pub id: Uuid,
    pub note_id: Uuid,
    /// Title at the time of snapshot (stored as string for historical purposes)
    pub title: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl NoteVersion {
    pub fn new(note_id: Uuid, title: Option<String>, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            note_id,
            title,
            content,
            created_at: Utc::now(),
        }
    }
}

/// A derived link between two notes, typically generated by semantic similarity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoteLink {
    pub source_note_id: Uuid,
    pub target_note_id: Uuid,
    /// Similarity score (0.0 to 1.0)
    pub score: f32,
    pub created_at: DateTime<Utc>,
}

impl NoteLink {
    pub fn new(source_note_id: Uuid, target_note_id: Uuid, score: f32) -> Self {
        Self {
            source_note_id,
            target_note_id,
            score,
            created_at: Utc::now(),
        }
    }
}

/// Filter options for querying notes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoteFilter {
    pub is_pinned: Option<bool>,
    pub is_archived: Option<bool>,
    pub tag_id: Option<Uuid>,
}

impl NoteFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pinned(mut self) -> Self {
        self.is_pinned = Some(true);
        self
    }

    pub fn archived(mut self) -> Self {
        self.is_archived = Some(true);
        self
    }

    pub fn not_archived(mut self) -> Self {
        self.is_archived = Some(false);
        self
    }

    pub fn with_tag(mut self, tag_id: Uuid) -> Self {
        self.tag_id = Some(tag_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod user_tests {
        use super::*;

        #[test]
        fn test_new_user_has_unique_id() {
            let email1 = Email::try_from("user1@example.com").unwrap();
            let email2 = Email::try_from("user2@example.com").unwrap();
            let user1 = User::new("subject1", email1);
            let user2 = User::new("subject2", email2);

            assert_ne!(user1.id, user2.id);
        }

        #[test]
        fn test_new_user_sets_fields_correctly() {
            let email = Email::try_from("test@example.com").unwrap();
            let user = User::new("oidc|123456", email);

            assert_eq!(user.subject, "oidc|123456");
            assert_eq!(user.email_str(), "test@example.com");
            assert!(user.password_hash.is_none());
        }

        #[test]
        fn test_new_local_user_sets_fields_correctly() {
            let email = Email::try_from("local@example.com").unwrap();
            let user = User::new_local(email, "hashed_secret");

            assert_eq!(user.subject, "local@example.com");
            assert_eq!(user.email_str(), "local@example.com");
            assert_eq!(user.password_hash, Some("hashed_secret".to_string()));
        }

        #[test]
        fn test_user_with_id_preserves_all_fields() {
            let id = Uuid::new_v4();
            let created_at = Utc::now();
            let email = Email::try_from("email@test.com").unwrap();
            let user = User::with_id(id, "subject", email, Some("hash".to_string()), created_at);

            assert_eq!(user.id, id);
            assert_eq!(user.subject, "subject");
            assert_eq!(user.email_str(), "email@test.com");
            assert_eq!(user.password_hash, Some("hash".to_string()));
            assert_eq!(user.created_at, created_at);
        }
    }

    mod tag_tests {
        use super::*;

        #[test]
        fn test_new_tag_has_unique_id() {
            let user_id = Uuid::new_v4();
            let name1 = TagName::try_from("work").unwrap();
            let name2 = TagName::try_from("personal").unwrap();
            let tag1 = Tag::new(name1, user_id);
            let tag2 = Tag::new(name2, user_id);

            assert_ne!(tag1.id, tag2.id);
        }

        #[test]
        fn test_new_tag_associates_with_user() {
            let user_id = Uuid::new_v4();
            let name = TagName::try_from("important").unwrap();
            let tag = Tag::new(name, user_id);

            assert_eq!(tag.user_id, user_id);
            assert_eq!(tag.name_str(), "important");
        }

        #[test]
        fn test_tag_with_id_preserves_all_fields() {
            let id = Uuid::new_v4();
            let user_id = Uuid::new_v4();
            let name = TagName::try_from("my-tag").unwrap();
            let tag = Tag::with_id(id, name, user_id);

            assert_eq!(tag.id, id);
            assert_eq!(tag.name_str(), "my-tag");
            assert_eq!(tag.user_id, user_id);
        }
    }

    mod note_tests {
        use super::*;

        #[test]
        fn test_new_note_has_unique_id() {
            let user_id = Uuid::new_v4();
            let title1 = NoteTitle::try_from("Title 1").ok();
            let title2 = NoteTitle::try_from("Title 2").ok();
            let note1 = Note::new(user_id, title1, "Content 1");
            let note2 = Note::new(user_id, title2, "Content 2");

            assert_ne!(note1.id, note2.id);
        }

        #[test]
        fn test_new_note_defaults() {
            let user_id = Uuid::new_v4();
            let title = NoteTitle::try_from("My Note").ok();
            let note = Note::new(user_id, title, "# Hello World");

            assert_eq!(note.user_id, user_id);
            assert_eq!(note.title_str(), "My Note");
            assert_eq!(note.content, "# Hello World");
            assert!(!note.is_pinned);
            assert!(!note.is_archived);
            assert!(note.tags.is_empty());
        }

        #[test]
        fn test_new_note_without_title() {
            let user_id = Uuid::new_v4();
            let note = Note::new(user_id, None, "Content without title");

            assert!(note.title.is_none());
            assert_eq!(note.title_str(), "");
            assert_eq!(note.content, "Content without title");
        }

        #[test]
        fn test_note_set_pinned_updates_timestamp() {
            let user_id = Uuid::new_v4();
            let title = NoteTitle::try_from("Title").ok();
            let mut note = Note::new(user_id, title, "Content");
            let original_updated_at = note.updated_at;

            // Small delay to ensure timestamp changes
            std::thread::sleep(std::time::Duration::from_millis(10));
            note.set_pinned(true);

            assert!(note.is_pinned);
            assert!(note.updated_at > original_updated_at);
        }

        #[test]
        fn test_note_set_archived_updates_timestamp() {
            let user_id = Uuid::new_v4();
            let title = NoteTitle::try_from("Title").ok();
            let mut note = Note::new(user_id, title, "Content");
            let original_updated_at = note.updated_at;

            std::thread::sleep(std::time::Duration::from_millis(10));
            note.set_archived(true);

            assert!(note.is_archived);
            assert!(note.updated_at > original_updated_at);
        }

        #[test]
        fn test_note_can_add_tag_when_under_limit() {
            let user_id = Uuid::new_v4();
            let title = NoteTitle::try_from("Title").ok();
            let note = Note::new(user_id, title, "Content");

            assert!(note.can_add_tag());
        }

        #[test]
        fn test_note_cannot_add_tag_when_at_limit() {
            let user_id = Uuid::new_v4();
            let title = NoteTitle::try_from("Title").ok();
            let mut note = Note::new(user_id, title, "Content");

            // Add MAX_TAGS_PER_NOTE tags
            for i in 0..MAX_TAGS_PER_NOTE {
                let tag_name = TagName::try_from(format!("tag-{}", i)).unwrap();
                note.tags.push(Tag::new(tag_name, user_id));
            }

            assert!(!note.can_add_tag());
            assert_eq!(note.tag_count(), MAX_TAGS_PER_NOTE);
        }

        #[test]
        fn test_note_set_title_updates_timestamp() {
            let user_id = Uuid::new_v4();
            let title = NoteTitle::try_from("Original").ok();
            let mut note = Note::new(user_id, title, "Content");
            let original_updated_at = note.updated_at;

            std::thread::sleep(std::time::Duration::from_millis(10));
            let new_title = NoteTitle::try_from("Updated Title").ok();
            note.set_title(new_title);

            assert_eq!(note.title_str(), "Updated Title");
            assert!(note.updated_at > original_updated_at);
        }

        #[test]
        fn test_note_set_content_updates_timestamp() {
            let user_id = Uuid::new_v4();
            let title = NoteTitle::try_from("Title").ok();
            let mut note = Note::new(user_id, title, "Original");
            let original_updated_at = note.updated_at;

            std::thread::sleep(std::time::Duration::from_millis(10));
            note.set_content("Updated content");

            assert_eq!(note.content, "Updated content");
            assert!(note.updated_at > original_updated_at);
        }
    }

    mod note_filter_tests {
        use super::*;

        #[test]
        fn test_default_filter_has_no_constraints() {
            let filter = NoteFilter::default();

            assert!(filter.is_pinned.is_none());
            assert!(filter.is_archived.is_none());
            assert!(filter.tag_id.is_none());
        }

        #[test]
        fn test_filter_builder_pattern() {
            let tag_id = Uuid::new_v4();
            let filter = NoteFilter::new().pinned().not_archived().with_tag(tag_id);

            assert_eq!(filter.is_pinned, Some(true));
            assert_eq!(filter.is_archived, Some(false));
            assert_eq!(filter.tag_id, Some(tag_id));
        }
    }
}
