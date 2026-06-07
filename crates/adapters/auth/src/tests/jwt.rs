use domain::user::{entity::User, value_objects::Email};

use crate::{config::JwtConfig, jwt::JwtValidator};

fn validator() -> JwtValidator {
    JwtValidator::new(JwtConfig::new(
        "a-test-secret-that-is-long-enough-for-hs256",
    ))
}

fn user() -> User {
    User::new_oidc("sub|123", Email::new("test@example.com").unwrap())
}

#[test]
fn create_and_validate_round_trip() {
    let v = validator();
    let u = user();
    let token = v.create_token(&u).unwrap();
    let claims = v.validate_token(&token).unwrap();

    assert_eq!(claims.email, "test@example.com");
    assert_eq!(claims.sub, u.id.as_uuid().to_string());
}

#[test]
fn wrong_secret_rejects_token() {
    let v1 = JwtValidator::new(JwtConfig::new(
        "secret-one-long-enough-for-hs256-validation",
    ));
    let v2 = JwtValidator::new(JwtConfig::new(
        "secret-two-long-enough-for-hs256-validation",
    ));

    let token = v1.create_token(&user()).unwrap();
    assert!(v2.validate_token(&token).is_err());
}

#[test]
fn invalid_token_is_rejected() {
    let v = validator();
    assert!(v.validate_token("not.a.valid.jwt").is_err());
}

#[test]
fn expired_token_returns_expired_error() {
    use crate::jwt::JwtError;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

    let secret = "a-test-secret-that-is-long-enough-for-hs256";
    let claims = crate::jwt::JwtClaims {
        sub: "user-id".into(),
        email: "x@example.com".into(),
        exp: 1, // epoch + 1 second — already expired
        iat: 0,
        iss: None,
        aud: None,
    };
    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();

    let v = JwtValidator::new(JwtConfig::new(secret));
    assert!(matches!(v.validate_token(&token), Err(JwtError::Expired)));
}
