use crate::helper::hash_password;
use crate::models::user::User;
use crate::paseto::AuthTokenClaims;
use crate::req_res::auth::{NewUser, RedactedUser};
use crate::req_res::me::UpdateUser;
use crate::req_res::users::{AdminNewUserReq, AdminUpdateUserReq};
use crate::req_res::AppError;
use crate::schema::private;
use crate::schema::private::users::uuid as SqlUuid;
use crate::utils::generate_random_string;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use log::error;
use pasetors::claims::Claims;
use std::sync::Arc;
use uuid::Uuid;

pub fn get_routes() -> Router<Arc<AppState>> {
    Router::new().nest(
        "/users/",
        Router::new()
            .route("/", get(get_users).post(create_user))
            .route(
                "/{id}",
                get(get_user).patch(update_user).delete(delete_user),
            )
            .route("/{id}/suspend", post(suspend_user))
            .route("/{id}/activate", post(unsuspend_user))
            .route("/{id}/reset-password", post(reset_password)),
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

    let redacted_vec = users_vec
        .into_iter()
        .map(|u| u.into())
        .collect::<Vec<RedactedUser>>();

    Ok((StatusCode::OK, Json(redacted_vec)))
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
        .optional()
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found())?;
    let redacted: RedactedUser = user.into();

    Ok((StatusCode::OK, Json(redacted)))
}

async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AdminNewUserReq>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let mut n_user: NewUser = payload.try_into()?;
    let random_password = generate_random_string();
    n_user.password = hash_password(&random_password)?;
    let created_user = diesel::insert_into(private::users::table)
        .values(&n_user)
        .get_result::<User>(&mut con)
        .await
        .map_err(AppError::from)?;
    let redacted: RedactedUser = created_user.into();
    //TODO: Send generated password via mail or text
    println!("created password: {}", random_password);
    Ok((StatusCode::CREATED, Json(redacted)))
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    Json(update_user): Json<AdminUpdateUserReq>,
) -> Result<impl IntoResponse, AppError> {
    let update_user: UpdateUser = update_user.try_into()?;

    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    diesel::update(private::users::table)
        .filter(SqlUuid.eq(uid))
        .set(&update_user)
        .get_result::<User>(&mut con)
        .await
        .optional()
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found())?;
    Ok((StatusCode::OK, ()))
}
async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    Extension(c): Extension<Option<Claims>>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let claims = c.ok_or_else(AppError::unauthorized)?;
    let claims = AuthTokenClaims::try_from(&claims).map_err(|err| {
        error!("Error parsing claims {}", err);
        AppError::unauthorized()
    })?;

    if claims.user_uid == uid {
        return Err(AppError::bad_request(None));
    }

    let deleted_count = diesel::delete(private::users::table.find(uid))
        .execute(&mut con)
        .await
        .map_err(AppError::from)?;

    if deleted_count == 0 {
        return Err(AppError::not_found());
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn reset_password(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    Extension(c): Extension<Option<Claims>>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let claims = c.ok_or_else(AppError::unauthorized)?;
    let claims = AuthTokenClaims::try_from(&claims).map_err(|err| {
        error!("Error parsing claims {}", err);
        AppError::unauthorized()
    })?;

    if claims.user_uid == uid {
        return Err(AppError::bad_request(None));
    }

    let random_password = generate_random_string();
    let hashed_password = hash_password(&random_password)?;

    let user = diesel::update(private::users::table)
        .filter(SqlUuid.eq(uid))
        .set(private::users::password.eq(hashed_password))
        .get_result::<User>(&mut con)
        .await
        .optional()
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found())?;

    //TODO: Send new password via email or text
    println!("New password: {}", random_password);

    Ok(StatusCode::OK)
}

async fn suspend_user(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    Extension(c): Extension<Option<Claims>>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let claims = c.ok_or_else(AppError::unauthorized)?;
    let claims = AuthTokenClaims::try_from(&claims).map_err(|err| {
        error!("Error parsing claims {}", err);
        AppError::unauthorized()
    })?;

    if claims.user_uid == uid {
        return Err(AppError::bad_request(None));
    }

    diesel::update(private::users::table)
        .filter(SqlUuid.eq(uid))
        .set(private::users::active.eq(false))
        .get_result::<User>(&mut con)
        .await
        .optional()
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found())?;

    Ok((StatusCode::OK, ()))
}

async fn unsuspend_user(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    Extension(c): Extension<Option<Claims>>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let claims = c.ok_or_else(AppError::unauthorized)?;
    let claims = AuthTokenClaims::try_from(&claims).map_err(|err| {
        error!("Error parsing claims {}", err);
        AppError::unauthorized()
    })?;

    if claims.user_uid == uid {
        return Err(AppError::bad_request(None));
    }

    diesel::update(private::users::table)
        .filter(SqlUuid.eq(uid))
        .set(private::users::active.eq(true))
        .get_result::<User>(&mut con)
        .await
        .optional()
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found())?;

    Ok((StatusCode::OK, ()))
}
