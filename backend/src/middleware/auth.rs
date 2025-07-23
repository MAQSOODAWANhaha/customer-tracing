use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: i32,
    pub username: String,
    pub name: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: i32,
    pub username: String,
    pub name: String,
}

impl From<Claims> for CurrentUser {
    fn from(claims: Claims) -> Self {
        Self {
            id: claims.user_id,
            username: claims.username,
            name: claims.name,
        }
    }
}

pub async fn auth_middleware<T>(
    State(app_state): State<T>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode>
where
    T: Clone + Send + Sync + 'static,
    T: AsRef<String>, // Assuming app_state has jwt_secret as String
{
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let jwt_secret = app_state.as_ref();
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let current_user = CurrentUser::from(claims.claims);
    request.extensions_mut().insert(current_user);
    
    Ok(next.run(request).await)
}