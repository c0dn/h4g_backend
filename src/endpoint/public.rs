use axum::{
    extract::Path,
    response::IntoResponse,
    http::{header, StatusCode, HeaderMap},
};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::path::PathBuf;
use crate::req_res::AppError;

pub async fn serve_upload(
    Path(file_path): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    // Sanitize and validate path
    let path = PathBuf::from("uploads").join(file_path);

    // Ensure path is within uploads directory
    let canonical_path = path.canonicalize()
        .map_err(|_| AppError::not_found())?;
    let uploads_dir = PathBuf::from("uploads")
        .canonicalize()
        .map_err(|_| AppError::internal_error("Unable to serve files".to_string()))?;

    if !canonical_path.starts_with(uploads_dir) {
        return Err(AppError::forbidden());
    }

    let mut file = File::open(&path)
        .await
        .map_err(|_| AppError::not_found())?;

    let mut contents = vec![];
    file.read_to_end(&mut contents)
        .await
        .map_err(|_| AppError::internal_error("Unable to read file".to_string()))?;

    // Set content type
    let mut headers = HeaderMap::new();
    if path.extension().and_then(|e| e.to_str()) == Some("webp") {
        headers.insert(header::CONTENT_TYPE, "image/webp".parse().unwrap());
    }

    Ok((StatusCode::OK, headers, contents))
}