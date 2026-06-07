use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use domain::{
    errors::{DomainError, DomainResult},
    events::{DomainEvent, EventPublisher},
    note::{
        entity::{Note, NoteFilter, NoteId, NoteLink, NoteVersion},
        ports::{LinkRepository, NoteRepository},
    },
    tag::{
        entity::{Tag, TagId},
        ports::TagRepository,
        value_objects::TagName,
    },
    user::{
        entity::{User, UserId},
        ports::{PasswordHasher, UserRepository},
        value_objects::{Email, Password, PasswordHash},
    },
};

use crate::{
    config::{AppConfig, SmartConfig},
    context::{AppContext, Repositories, Services},
};

// ── Note ─────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MemoryNoteRepo {
    notes: Mutex<HashMap<NoteId, Note>>,
    versions: Mutex<HashMap<NoteId, Vec<NoteVersion>>>,
}

#[async_trait]
impl NoteRepository for MemoryNoteRepo {
    async fn find_by_id(&self, id: &NoteId) -> DomainResult<Option<Note>> {
        Ok(self.notes.lock().unwrap().get(id).cloned())
    }

    async fn find_by_user(&self, user_id: &UserId, filter: NoteFilter) -> DomainResult<Vec<Note>> {
        Ok(self
            .notes
            .lock()
            .unwrap()
            .values()
            .filter(|n| {
                n.user_id == *user_id
                    && filter.is_pinned.map_or(true, |v| n.is_pinned == v)
                    && filter.is_archived.map_or(true, |v| n.is_archived == v)
            })
            .cloned()
            .collect())
    }

    async fn search(&self, user_id: &UserId, query: &str) -> DomainResult<Vec<Note>> {
        let q = query.to_lowercase();
        Ok(self
            .notes
            .lock()
            .unwrap()
            .values()
            .filter(|n| {
                n.user_id == *user_id
                    && (n.content.to_lowercase().contains(&q)
                        || n.title
                            .as_ref()
                            .map_or(false, |t| t.as_ref().to_lowercase().contains(&q)))
            })
            .cloned()
            .collect())
    }

    async fn save(&self, note: &Note) -> DomainResult<()> {
        self.notes.lock().unwrap().insert(note.id, note.clone());
        Ok(())
    }

    async fn delete(&self, id: &NoteId) -> DomainResult<()> {
        self.notes.lock().unwrap().remove(id);
        Ok(())
    }

    async fn save_version(&self, v: &NoteVersion) -> DomainResult<()> {
        self.versions
            .lock()
            .unwrap()
            .entry(v.note_id)
            .or_default()
            .push(v.clone());
        Ok(())
    }

    async fn find_versions(&self, note_id: &NoteId) -> DomainResult<Vec<NoteVersion>> {
        Ok(self
            .versions
            .lock()
            .unwrap()
            .get(note_id)
            .cloned()
            .unwrap_or_default())
    }
}

// ── Tag ──────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MemoryTagRepo {
    tags: Mutex<HashMap<TagId, Tag>>,
    note_tags: Mutex<HashMap<(NoteId, TagId), ()>>,
}

#[async_trait]
impl TagRepository for MemoryTagRepo {
    async fn find_by_id(&self, id: &TagId) -> DomainResult<Option<Tag>> {
        Ok(self.tags.lock().unwrap().get(id).cloned())
    }

    async fn find_by_user(&self, user_id: &UserId) -> DomainResult<Vec<Tag>> {
        Ok(self
            .tags
            .lock()
            .unwrap()
            .values()
            .filter(|t| t.user_id == *user_id)
            .cloned()
            .collect())
    }

    async fn find_by_name(&self, user_id: &UserId, name: &TagName) -> DomainResult<Option<Tag>> {
        Ok(self
            .tags
            .lock()
            .unwrap()
            .values()
            .find(|t| t.user_id == *user_id && t.name == *name)
            .cloned())
    }

    async fn find_by_note(&self, note_id: &NoteId) -> DomainResult<Vec<Tag>> {
        let note_tags = self.note_tags.lock().unwrap();
        let tags = self.tags.lock().unwrap();
        Ok(note_tags
            .keys()
            .filter(|(nid, _)| nid == note_id)
            .filter_map(|(_, tid)| tags.get(tid).cloned())
            .collect())
    }

    async fn save(&self, tag: &Tag) -> DomainResult<()> {
        self.tags.lock().unwrap().insert(tag.id, tag.clone());
        Ok(())
    }

    async fn delete(&self, id: &TagId) -> DomainResult<()> {
        self.tags.lock().unwrap().remove(id);
        Ok(())
    }

    async fn add_to_note(&self, tag_id: &TagId, note_id: &NoteId) -> DomainResult<()> {
        self.note_tags
            .lock()
            .unwrap()
            .insert((*note_id, *tag_id), ());
        Ok(())
    }

    async fn remove_from_note(&self, tag_id: &TagId, note_id: &NoteId) -> DomainResult<()> {
        self.note_tags.lock().unwrap().remove(&(*note_id, *tag_id));
        Ok(())
    }
}

// ── User ─────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MemoryUserRepo {
    users: Mutex<HashMap<UserId, User>>,
}

#[async_trait]
impl UserRepository for MemoryUserRepo {
    async fn find_by_id(&self, id: &UserId) -> DomainResult<Option<User>> {
        Ok(self.users.lock().unwrap().get(id).cloned())
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

    async fn find_by_email(&self, email: &Email) -> DomainResult<Option<User>> {
        Ok(self
            .users
            .lock()
            .unwrap()
            .values()
            .find(|u| u.email.as_ref() == email.as_ref())
            .cloned())
    }

    async fn save(&self, user: &User) -> DomainResult<()> {
        self.users.lock().unwrap().insert(user.id, user.clone());
        Ok(())
    }

    async fn delete(&self, id: &UserId) -> DomainResult<()> {
        self.users.lock().unwrap().remove(id);
        Ok(())
    }
}

// ── Link ─────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct MemoryLinkRepo {
    links: Mutex<Vec<NoteLink>>,
}

#[async_trait]
impl LinkRepository for MemoryLinkRepo {
    async fn save_links(&self, links: &[NoteLink]) -> DomainResult<()> {
        self.links.lock().unwrap().extend_from_slice(links);
        Ok(())
    }

    async fn delete_for_source(&self, source_id: &NoteId) -> DomainResult<()> {
        self.links
            .lock()
            .unwrap()
            .retain(|l| l.source_id != *source_id);
        Ok(())
    }

    async fn find_for_note(&self, note_id: &NoteId) -> DomainResult<Vec<NoteLink>> {
        Ok(self
            .links
            .lock()
            .unwrap()
            .iter()
            .filter(|l| l.source_id == *note_id)
            .cloned()
            .collect())
    }
}

// ── PasswordHasher ───────────────────────────────────────────────────────────

pub struct PlaintextHasher;

#[async_trait]
impl PasswordHasher for PlaintextHasher {
    async fn hash(&self, password: &Password) -> DomainResult<PasswordHash> {
        Ok(PasswordHash::new(format!("hashed:{}", password.as_ref())))
    }

    async fn verify(&self, password: &Password, hash: &PasswordHash) -> DomainResult<bool> {
        Ok(hash.as_str() == format!("hashed:{}", password.as_ref()))
    }
}

// ── EventPublisher ───────────────────────────────────────────────────────────

#[derive(Default)]
pub struct RecordingPublisher {
    pub events: Mutex<Vec<DomainEvent>>,
}

#[async_trait]
impl EventPublisher for RecordingPublisher {
    async fn publish(&self, event: &DomainEvent) -> Result<(), DomainError> {
        self.events.lock().unwrap().push(event.clone());
        Ok(())
    }
}

// ── AppContext builder ────────────────────────────────────────────────────────

pub struct TestContext {
    pub ctx: AppContext,
    pub publisher: Arc<RecordingPublisher>,
}

impl TestContext {
    pub fn new() -> Self {
        use domain::events::EventConsumer;
        use futures::stream::BoxStream;

        struct NoopConsumer;
        impl EventConsumer for NoopConsumer {
            fn consume(&self) -> BoxStream<'_, Result<domain::events::EventEnvelope, DomainError>> {
                Box::pin(futures::stream::empty())
            }
        }

        let publisher = Arc::new(RecordingPublisher::default());

        let ctx = AppContext {
            repos: Repositories {
                note: Arc::new(MemoryNoteRepo::default()),
                tag: Arc::new(MemoryTagRepo::default()),
                user: Arc::new(MemoryUserRepo::default()),
                link: Arc::new(MemoryLinkRepo::default()),
            },
            services: Services {
                password_hasher: Arc::new(PlaintextHasher),
                event_publisher: Arc::clone(&publisher) as Arc<dyn EventPublisher>,
                embedding: None,
                vector_store: None,
                event_consumer: Arc::new(NoopConsumer),
            },
            config: AppConfig {
                base_url: "http://localhost:3000".into(),
                smart: SmartConfig::default(),
                allow_registration: true,
            },
        };

        Self { ctx, publisher }
    }
}
