use std::sync::LazyLock;
use std::time::Duration;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use moka::sync::Cache;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::models::UserRole;

static JWT_SECRET_LAZY: LazyLock<Vec<u8>> = LazyLock::new(|| {
    std::env::var("JWT_SECRET")
        .map(|s| s.into_bytes())
        .unwrap_or_else(|_| b"topcoat_library_application_jwt_secret_key_2026_super_secure!".to_vec())
});

fn get_jwt_secret() -> &'static [u8] {
    &JWT_SECRET_LAZY
}

pub const AUTH_COOKIE_NAME: &str = "auth_token";
pub const JWT_EXPIRATION_HOURS: i64 = 8;

static TOKEN_CACHE: LazyLock<Cache<String, AuthenticatedUser>> = LazyLock::new(|| {
    Cache::builder()
        .max_capacity(5_000)
        .time_to_idle(Duration::from_secs(60))
        .build()
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: User primary key (as string representation)
    pub sub: String,
    /// Username for display in layout header
    pub username: String,
    /// Role for authorization
    pub role: UserRole,
    /// Issued At (UNIX epoch seconds)
    pub iat: i64,
    /// Expiration time (UNIX epoch seconds)
    pub exp: i64,
    /// Unique Token Identifier
    pub jti: String,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: i64,
    pub username: String,
    pub role: UserRole,
}

pub fn create_jwt(user_id: i64, username: &str, role: UserRole) -> Result<String, String> {
    let now = Utc::now().timestamp();
    let exp = now + (JWT_EXPIRATION_HOURS * 3600);

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        role,
        iat: now,
        exp,
        jti: Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret()),
    )
    .map_err(|e| format!("JWT generation failed: {e}"))
}

pub fn verify_jwt(token: &str) -> Result<AuthenticatedUser, String> {
    if let Some(cached) = TOKEN_CACHE.get(token) {
        return Ok(cached);
    }

    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret()),
        &validation,
    )
    .map_err(|e| format!("Invalid or expired token: {e}"))?;

    let claims = token_data.claims;
    let user_id = claims
        .sub
        .parse::<i64>()
        .map_err(|_| "Malformed user ID in token sub claim".to_string())?;

    let auth_user = AuthenticatedUser {
        id: user_id,
        username: claims.username,
        role: claims.role,
    };

    TOKEN_CACHE.insert(token.to_string(), auth_user.clone());
    Ok(auth_user)
}
