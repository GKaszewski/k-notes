use domain::user::{
    ports::PasswordHasher,
    value_objects::{Password, PasswordHash},
};

use crate::password::Argon2PasswordHasher;

#[tokio::test]
async fn hash_produces_verifiable_hash() {
    let hasher = Argon2PasswordHasher;
    let password = Password::new("correcthorsebattery").unwrap();

    let hash = hasher.hash(&password).await.unwrap();
    assert!(hasher.verify(&password, &hash).await.unwrap());
}

#[tokio::test]
async fn wrong_password_does_not_verify() {
    let hasher = Argon2PasswordHasher;
    let password = Password::new("correcthorsebattery").unwrap();
    let wrong = Password::new("wrongpassword12345").unwrap();

    let hash = hasher.hash(&password).await.unwrap();
    assert!(!hasher.verify(&wrong, &hash).await.unwrap());
}

#[tokio::test]
async fn same_password_produces_different_hashes() {
    let hasher = Argon2PasswordHasher;
    let password = Password::new("samepassword123").unwrap();

    let hash1 = hasher.hash(&password).await.unwrap();
    let hash2 = hasher.hash(&password).await.unwrap();

    assert_ne!(hash1.as_str(), hash2.as_str());
}
