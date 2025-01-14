use crate::helper::hash_password;
use crate::models::user::User;
use crate::paseto::AuthTokenClaims;
use crate::req_res::me::{PasswordChangeReq, PasswordChangeValidated};
use crate::req_res::AppError;
use crate::schema::private;
use crate::schema::private::users::uuid as SqlUuid;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use log::error;
use pasetors::claims::Claims;
use serde_json::json;
use std::sync::Arc;

pub fn get_scope() -> Router<Arc<AppState>> {
    Router::new()
        .route("/change-required", get(check_pw_change))
        .route("/settings", get(settings))
        .route("/settings/change-password", post(process_password_change))
}

async fn settings(State(_state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, ()))
}

async fn check_pw_change(
    State(state): State<Arc<AppState>>,
    Extension(c): Extension<Option<Claims>>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let claims = c.ok_or_else(AppError::unauthorized)?;
    let claims = AuthTokenClaims::try_from(&claims).map_err(|err| {
        error!("Error parsing claims {}", err);
        AppError::unauthorized()
    })?;

    let user = private::users::table
        .filter(private::users::uuid.eq(claims.user_uid))
        .first::<User>(&mut con)
        .await
        .optional()?
        .ok_or_else(AppError::unauthorized)?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "change_required": user.force_pw_change
        })),
    ))
}

async fn process_password_change(
    State(state): State<Arc<AppState>>,
    Extension(c): Extension<Option<Claims>>,
    Json(payload): Json<PasswordChangeReq>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let claims = c.ok_or_else(AppError::unauthorized)?;
    let claims = AuthTokenClaims::try_from(&claims).map_err(|err| {
        error!("Error parsing claims {}", err);
        AppError::unauthorized()
    })?;
    let req: PasswordChangeValidated = payload.try_into()?;
    let hashed_password = hash_password(&req.password)?;

    diesel::update(private::users::table)
        .filter(private::users::uuid.eq(claims.user_uid))
        .set((
            private::users::password.eq(hashed_password),
            private::users::force_pw_change.eq(false),
        ))
        .execute(&mut con)
        .await?;

    Ok((StatusCode::OK, ()))
}
