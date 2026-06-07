use super::*;

#[test]
fn email_normalises() {
    let e = Email::new("  USER@EXAMPLE.COM  ").unwrap();
    assert_eq!(e.as_ref(), "user@example.com");
}

#[test]
fn email_rejects_invalid() {
    assert!(Email::new("not-an-email").is_err());
    assert!(Email::new("@example.com").is_err());
}

#[test]
fn password_enforces_minimum_length() {
    assert!(Password::new("short").is_err());
    assert!(Password::new("longenough").is_ok());
}

#[test]
fn password_hides_in_debug() {
    let p = Password::new("supersecret").unwrap();
    assert!(!format!("{p:?}").contains("supersecret"));
}
