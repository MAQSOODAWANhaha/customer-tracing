use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use tower_http::{
    cors::{CorsLayer, Any},
    services::ServeDir,
};

use crate::{
    handlers::{auth, customer, customer_track},
    middleware::auth::auth_middleware,
    handlers::auth::AppState,
};

pub fn create_routes(app_state: AppState) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::login))
        .route("/api/health", get(health_check))
        .with_state(app_state.clone());

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/api/auth/me", get(auth::get_current_user))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/refresh", post(auth::refresh_token))
        
        // Customer routes
        .route("/api/customers", 
            get(customer::list_customers)
            .post(customer::create_customer)
        )
        .route("/api/customers/{id}", 
            get(customer::get_customer)
            .put(customer::update_customer)
            .delete(customer::delete_customer)
        )
        
        // Customer tracking routes
        .route("/api/customers/{id}/tracks", 
            get(customer_track::list_customer_tracks)
            .post(customer_track::create_customer_track)
        )
        .route("/api/tracks", get(customer_track::list_tracks).post(customer_track::create_track))
        .route("/api/tracks/{id}", 
            put(customer_track::update_customer_track)
            .delete(customer_track::delete_customer_track)
        )
        .route("/api/tracks/actions", get(customer_track::get_next_actions))
        
        .layer(middleware::from_fn_with_state(
            app_state.clone(), 
            auth_middleware::<AppState>
        ))
        .with_state(app_state.clone());

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(create_cors_layer(&app_state))
        .fallback_service(ServeDir::new("dist")) // Serve static files
}

fn create_cors_layer(_app_state: &AppState) -> CorsLayer {
    // 极简模式：默认允许所有来源，简化部署配置
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

async fn health_check() -> &'static str {
    "OK"
}