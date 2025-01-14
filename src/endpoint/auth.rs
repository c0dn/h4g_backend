use crate::backend::pw_reset::{
    new_password_reset_req, verify_password_reset_otp, verify_reset_token, PasswordResetReq,
};
use crate::helper::{hash_password, is_bad_mail, validate_token, verify_password};
use crate::models::user::User;
use crate::paseto::AuthTokenClaims;
use crate::req_res::auth::{
    AppInitRequest, NewTokens, NewUser, PasswordResetOtpReq, PasswordResetRequest,
    PasswordResetRes, PwResetOtpValidated, ResetParams, UserAuthRequest,
    UserAuthenticationResponse,
};
use crate::req_res::me::{PasswordChangeReq, PasswordChangeValidated};
use crate::req_res::{AppError, ClientErrorMessages, DataValidationError};
use crate::schema::private;
use crate::schema::private::users::dsl::users;
use crate::schema::private::users::resident_id;
use crate::schema::private::users::uuid as SqlUuid;
use crate::utils::generate_otp;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use chrono::{Duration, Utc};
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use log::{error, warn};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub fn get_scope() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/init", get(check_init_state))
        .route("/init", post(init_app))
        .route("/password-reset", post(initiate_password_reset))
        .route("/password-reset/otp", post(verify_pw_otp))
        .route("/password-reset/{uid}", post(complete_pw_reset))
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserAuthRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let mut con = pool.get().await?;
    let user_result = users
        .filter(resident_id.eq(&payload.resident_id))
        .first::<User>(&mut con)
        .await;

    match user_result {
        Ok(user) => {
            verify_password(&user.password, &payload.password)?;
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

async fn initiate_password_reset(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PasswordResetRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let redis = &state.redis_client;
    let mut conn = pool.get().await?;
    let matched_user: Option<User> = private::users::table
        .filter(private::users::phone.eq(&payload.phone))
        .filter(private::users::active.eq(true))
        .first(&mut conn)
        .await
        .optional()?;

    let session_uid = Uuid::new_v4();
    let expire = Utc::now() + Duration::minutes(10);
    let res = PasswordResetRes {
        session_uid: session_uid.to_string(),
        message: format!("OTP sent to {}", payload.phone),
        otp_sent: true,
        otp_expiry: Some(expire),
    };

    if let Some(matched_user) = matched_user {
        let otp = generate_otp();
        println!("pw reset otp: {}", &otp);
        new_password_reset_req(redis, session_uid, &otp, expire.clone(), matched_user.uuid).await?;
        Ok((StatusCode::OK, Json(res)))
    } else {
        warn!("Invalid user");
        Ok((StatusCode::OK, Json(res)))
    }
}

async fn verify_pw_otp(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PasswordResetOtpReq>,
) -> Result<impl IntoResponse, AppError> {
    let redis = &state.redis_client;
    let req: PwResetOtpValidated = payload.try_into()?;
    let result = verify_password_reset_otp(redis, req.session_uid, &req.otp).await?;
    Ok((StatusCode::OK, Json(result)))
}

async fn complete_pw_reset(
    State(state): State<Arc<AppState>>,
    Path(uid): Path<Uuid>,
    Query(params): Query<ResetParams>,
    Json(payload): Json<PasswordChangeReq>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.postgres_pool;
    let redis = &state.redis_client;
    let mut conn = pool.get().await?;
    let req: PasswordChangeValidated = payload.try_into()?;
    let user_uuid = verify_reset_token(redis, uid, &params.token).await?;
    match user_uuid {
        Some(uuid) => {
            let hashed_password = hash_password(&req.password)?;

            diesel::update(private::users::table)
                .filter(private::users::uuid.eq(uuid))
                .set((
                    private::users::password.eq(hashed_password),
                    private::users::force_pw_change.eq(false),
                ))
                .execute(&mut conn)
                .await?;

            Ok((StatusCode::OK, ()))
        }
        None => Ok((StatusCode::BAD_REQUEST, ())),
    }
}
