use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use crate::middleware::auth::Claims;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub expires_in: i64,
}

pub fn generate_jwt_token(
    user_id: i32,
    username: &str,
    name: &str,
    secret: &str,
    expires_hours: i64,
) -> Result<TokenPair, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(expires_hours);
    
    let claims = Claims {
        user_id,
        username: username.to_string(),
        name: name.to_string(),
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )?;

    Ok(TokenPair {
        access_token: token,
        expires_in: expires_hours * 3600, // Convert to seconds
    })
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{decode, DecodingKey, Validation};
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;
    
    Ok(token_data.claims)
}