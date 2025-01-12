use crate::models::user::User;
use crate::req_res::AppError;
use crate::schema::private;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router};
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use pasetors::claims::Claims;
use crate::req_res::auth::NewUser;
use crate::req_res::me::UpdateUser;
use crate::req_res::users::{AdminNewUserReq, AdminUpdateUserReq};
use crate::schema::private::users::uuid as SqlUuid;
use uuid::Uuid;

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/users/",
        Router::new()
            .route("/", get(get_users).post(create_user))
            .route("/{id}", get(get_user).put(update_user).delete(delete_user)),
    )
}

async fn get_users(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let users_vec = private::users::table
        .select(User::as_select())
        .load::<User>(&mut con)
        .await
        .map_err(AppError::from)?;

    Ok((StatusCode::OK, Json(users_vec)))
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let user = private::users::table
        .filter(SqlUuid.eq(uid))
        .first::<User>(&mut con)
        .await
        .map_err(AppError::from)?;

    Ok((StatusCode::OK, Json(user)))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminNewUserReq>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let n_user: NewUser = payload.try_into()?;
    let created_user = diesel::insert_into(private::users::table)
        .values(&n_user)
        .get_result::<User>(&mut con)
        .await
        .map_err(AppError::from)?;

    Ok((StatusCode::CREATED, Json(created_user)))
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    Json(update_user): Json<AdminUpdateUserReq>,
) -> Result<impl IntoResponse, AppError> {
    let update_user: UpdateUser = update_user.try_into()?;

    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let user = diesel::update(private::users::table)
        .filter(SqlUuid.eq(uid))
        .set(&update_user)
        .get_result::<User>(&mut con)
        .await
        .map_err(AppError::from)?;
    Ok((StatusCode::OK, Json(user)))
}
async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    Extension(claims): Extension<Option<Claims>>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    diesel::delete(private::users::table.find(uid))
        .execute(&mut con)
        .await
        .map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
