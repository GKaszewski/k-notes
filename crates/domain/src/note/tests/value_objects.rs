use super::*;

#[test]
fn title_trims_whitespace() {
    let t = NoteTitle::new("  My Note  ").unwrap();
    assert_eq!(t.as_ref(), "My Note");
}

#[test]
fn title_rejects_too_long() {
    assert!(NoteTitle::new("a".repeat(MAX_NOTE_TITLE_LENGTH + 1)).is_err());
    assert!(NoteTitle::new("a".repeat(MAX_NOTE_TITLE_LENGTH)).is_ok());
}

#[test]
fn title_from_optional_empty_is_none() {
    assert!(NoteTitle::from_optional(None).unwrap().is_none());
    assert!(
        NoteTitle::from_optional(Some("  ".into()))
            .unwrap()
            .is_none()
    );
}

#[test]
fn color_uppercases() {
    let c = NoteColor::new("default");
    assert_eq!(c.as_str(), "DEFAULT");
}

#[test]
fn color_default() {
    assert_eq!(NoteColor::default().as_str(), "DEFAULT");
}
