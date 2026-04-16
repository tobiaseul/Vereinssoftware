use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AdminRole { Admin, SuperAdmin }

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: AdminRole,
    pub exp: i64,
}

pub fn create_access_token(admin_id: Uuid, role: AdminRole, secret: &str, expiry_seconds: u64) -> String {
    let exp = (Utc::now() + Duration::seconds(expiry_seconds as i64)).timestamp();
    let claims = Claims { sub: admin_id, role, exp };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap()
}

pub fn validate_access_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())?;
    Ok(data.claims)
}

pub fn hash_password(password: &str) -> String {
    use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default().hash_password(password.as_bytes(), &salt).unwrap().to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{password_hash::{PasswordHash, PasswordVerifier}, Argon2};
    let parsed = PasswordHash::new(hash).unwrap();
    Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

pub fn generate_refresh_token() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

pub fn hash_refresh_token(token: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_round_trip() {
        let id = Uuid::new_v4();
        let token = create_access_token(id, AdminRole::SuperAdmin, "secret", 900);
        let claims = validate_access_token(&token, "secret").unwrap();
        assert_eq!(claims.sub, id);
        assert_eq!(claims.role, AdminRole::SuperAdmin);
    }

    #[test]
    fn test_invalid_jwt_rejected() {
        let result = validate_access_token("invalid.token.here", "secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hash_and_verify() {
        let hash = hash_password("hunter2");
        assert!(verify_password("hunter2", &hash));
        assert!(!verify_password("wrong", &hash));
    }
}
