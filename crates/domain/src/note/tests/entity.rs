use crate::{
    note::entity::{MAX_TAGS_PER_NOTE, Note, NoteVersion},
    tag::{entity::Tag, value_objects::TagName},
    user::entity::UserId,
};

fn uid() -> UserId {
    UserId::new()
}

#[test]
fn new_note_defaults() {
    let note = Note::new(uid(), None, "content");
    assert!(!note.is_pinned);
    assert!(!note.is_archived);
    assert_eq!(note.color.as_str(), "DEFAULT");
    assert!(note.tags.is_empty());
}

#[test]
fn set_pinned_updates_timestamp() {
    let mut note = Note::new(uid(), None, "content");
    let before = note.updated_at;
    std::thread::sleep(std::time::Duration::from_millis(5));
    note.set_pinned(true);
    assert!(note.is_pinned);
    assert!(note.updated_at > before);
}

#[test]
fn can_add_tag_respects_limit() {
    let user_id = uid();
    let mut note = Note::new(user_id, None, "content");
    assert!(note.can_add_tag());
    note.tags = (0..MAX_TAGS_PER_NOTE)
        .map(|_| Tag::new(TagName::new("x").unwrap(), user_id))
        .collect();
    assert!(!note.can_add_tag());
}

#[test]
fn note_version_snapshots_content() {
    let note = Note::new(uid(), None, "hello");
    let v = NoteVersion::snapshot(&note);
    assert_eq!(v.note_id, note.id);
    assert_eq!(v.content, "hello");
}
