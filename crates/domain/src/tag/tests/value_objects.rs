use super::*;

#[test]
fn normalises_to_lowercase_trimmed() {
    let t = TagName::new("  Important  ").unwrap();
    assert_eq!(t.as_ref(), "important");
}

#[test]
fn rejects_empty() {
    assert!(TagName::new("").is_err());
    assert!(TagName::new("   ").is_err());
}

#[test]
fn rejects_too_long() {
    assert!(TagName::new("a".repeat(MAX_TAG_NAME_LENGTH + 1)).is_err());
    assert!(TagName::new("a".repeat(MAX_TAG_NAME_LENGTH)).is_ok());
}
