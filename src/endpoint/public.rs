use crate::req_res::AppError;
use axum::body::Body;
use axum::{
    extract::Path,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::path::PathBuf;
use tokio::fs::File;

pub async fn serve_upload(Path(file_path): Path<String>) -> Result<impl IntoResponse, AppError> {
    if file_path.contains("..") || file_path.starts_with('/') || file_path.contains('\\') {
        return Err(AppError::forbidden());
    }

    let uploads_dir = PathBuf::from("uploads")
        .canonicalize()
        .map_err(|_| AppError::internal_error("Unable to access uploads directory".to_string()))?;

    let path = uploads_dir.join(file_path);

    let canonical_path = path.canonicalize().map_err(|_| AppError::not_found())?;

    if !canonical_path.starts_with(&uploads_dir) {
        return Err(AppError::forbidden());
    }

    let file = File::open(&canonical_path)
        .await
        .map_err(|_| AppError::not_found())?;

    let mut headers = HeaderMap::new();
    if canonical_path.extension().and_then(|e| e.to_str()) == Some("webp") {
        headers.insert(header::CONTENT_TYPE, "image/webp".parse().unwrap());
    }

    let stream = tokio_util::io::ReaderStream::new(file);

    Ok((StatusCode::OK, headers, Body::from_stream(stream)))
}
