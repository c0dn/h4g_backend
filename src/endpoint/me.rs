use std::sync::Arc;
use crate::req_res::AppError;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;

pub fn get_scope() -> Router<Arc<AppState>> {
    Router::new().route("/settings", get(settings))
}

async fn settings(State(_state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, ()))
}
