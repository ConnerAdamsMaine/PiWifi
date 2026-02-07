use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // username
    pub exp: i64,         // expiration
    pub iat: i64,         // issued at
    pub role: String,     // "admin", "user", "viewer"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

pub struct AuthManager {
    secret: Arc<String>,
}

impl AuthManager {
    pub fn new(secret: String) -> Self {
        Self {
            secret: Arc::new(secret),
        }
    }

    /// Create a new JWT token
    pub fn create_token(&self, username: &str, role: &str) -> Result<AuthResponse> {
        let now = Utc::now();
        let exp = (now + Duration::hours(24)).timestamp();
        let iat = now.timestamp();

        let claims = Claims {
            sub: username.to_string(),
            exp,
            iat,
            role: role.to_string(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;

        Ok(AuthResponse {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: 86400, // 24 hours
        })
    }

    /// Verify and decode a JWT token
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        decode(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| anyhow!("Token validation failed: {}", e))
    }

    /// Hash a password using bcrypt
    pub fn hash_password(password: &str) -> Result<String> {
        bcrypt::hash(password, 12).map_err(|e| anyhow!("Password hashing failed: {}", e))
    }

    /// Verify a password against its hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        bcrypt::verify(password, hash).map_err(|e| anyhow!("Password verification failed: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation_and_verification() {
        let auth = AuthManager::new("test-secret".to_string());
        let response = auth.create_token("admin", "admin").unwrap();
        assert!(!response.access_token.is_empty());

        let claims = auth.verify_token(&response.access_token).unwrap();
        assert_eq!(claims.sub, "admin");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_password_hashing() {
        let password = "secure_password_123";
        let hash = AuthManager::hash_password(password).unwrap();
        assert!(AuthManager::verify_password(password, &hash).unwrap());
        assert!(!AuthManager::verify_password("wrong_password", &hash).unwrap());
    }
}
