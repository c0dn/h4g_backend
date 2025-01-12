use crate::helper::{is_bad_mail, validate_token, verify_password_phone};
use crate::models::user::User;
use crate::paseto::AuthTokenClaims;
use crate::req_res::auth::{
    AppInitRequest, NewTokens, NewUser, UserAuthRequest, UserAuthenticationResponse,
};
use crate::req_res::{AppError, ClientErrorMessages, DataValidationError};
use crate::schema::private::users::dsl::users;
use crate::schema::private::users::username;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use std::sync::Arc;

pub fn get_scope() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/init", get(check_init_state))
        .route("/init", post(init_app))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserAuthRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let user_result = users
        .filter(username.eq(&payload.username))
        .first::<User>(&mut con)
        .await;

    match user_result {
        Ok(user) => {
            verify_password_phone(&user.password, &payload.password)?;
            let res: UserAuthenticationResponse = user.into();
            Ok((StatusCode::OK, Json(res)))
        }
        Err(e) => match e {
            Error::NotFound => Err(AppError::unauthorized()),
            _ => Err(AppError::from(e)),
        },
    }
}

async fn check_init_state(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let user_count: i64 = users::table().count().get_result(&mut con).await?;

    if user_count == 0 {
        Ok((StatusCode::OK, ()))
    } else {
        Err(AppError::method_not_allowed())
    }
}

async fn init_app(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AppInitRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut conn = pool.get().await?;
    let user_count: i64 = users::table().count().get_result(&mut conn).await?;

    if user_count == 0 {
        if is_bad_mail(&payload.email).await {
            let errors = vec![
                "Invalid email, make sure you are not using a temp email provider".to_string(),
            ];
            Err(AppError::bad_request::<ClientErrorMessages>(
                DataValidationError { errors }.into(),
            ))?
        } else {
            let n_user: NewUser = payload.try_into()?;
            let created_user = diesel::insert_into(users::table())
                .values(n_user)
                .returning(User::as_returning())
                .get_result(&mut conn)
                .await?;
            let res: UserAuthenticationResponse = created_user.into();

            Ok((StatusCode::OK, Json(res)))
        }
    } else {
        Err(AppError::method_not_allowed())
    }
}

async fn refresh_token(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, AppError> {
    let (_, claims) = validate_token(bearer.token()).ok_or(AppError::unauthorized())?;
    let claims = AuthTokenClaims::try_from(&claims).map_err(|_| AppError::unauthorized())?;
    let res = NewTokens::new(claims.user_uid, claims.role);
    Ok((StatusCode::OK, Json(res)))
}
