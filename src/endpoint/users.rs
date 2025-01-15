use crate::helper::hash_password;
use crate::models::user::User;
use crate::models::wallet::Wallet;
use crate::paseto::AuthTokenClaims;
use crate::req_res::auth::{NewUser, RedactedUser};
use crate::req_res::me::UpdateUser;
use crate::req_res::users::{AdminNewUserReq, AdminUpdateUserReq, DetailedUser, DetailedUserFull};
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
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
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

    let users_with_wallets = private::users::table
        .left_join(private::wallets::table)
        .select((User::as_select(), Option::<Wallet>::as_select()))
        .load::<(User, Option<Wallet>)>(&mut con)
        .await
        .map_err(AppError::from)?;

    let detailed_vec = users_with_wallets
        .into_iter()
        .map(|tuple| tuple.into())
        .collect::<Vec<DetailedUser>>();

    Ok((StatusCode::OK, Json(detailed_vec)))
}
async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;

    let (user, wallet) = private::users::table
        .left_join(private::wallets::table)
        .filter(SqlUuid.eq(uid))
        .select((User::as_select(), Option::<Wallet>::as_select()))
        .first::<(User, Option<Wallet>)>(&mut con)
        .await
        .optional()
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::not_found())?;

    let detailed: DetailedUserFull = (user, wallet).into();

    Ok((StatusCode::OK, Json(detailed)))
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
    let (created_user, user_wallet) = con
        .transaction(|conn| {
            async move {
                let user = diesel::insert_into(private::users::table)
                    .values(&n_user)
                    .get_result::<User>(conn)
                    .await?;

                let wallet = diesel::insert_into(private::wallets::table)
                    .values((
                        private::wallets::user_uuid.eq(user.uuid),
                        private::wallets::balance.eq(0),
                    ))
                    .get_result::<Wallet>(conn)
                    .await?;

                Ok::<(User, Wallet), diesel::result::Error>((user, wallet))
            }
            .scope_boxed()
        })
        .await
        .map_err(AppError::from)?;

    let detailed: DetailedUser = (created_user, Some(user_wallet)).into();
    //TODO: Send generated password via mail or text
    println!("created password: {}", random_password);
    Ok((StatusCode::CREATED, Json(detailed)))
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
