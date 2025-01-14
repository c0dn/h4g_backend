use crate::AppState;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new().nest("/inventory/", Router::new())
}
