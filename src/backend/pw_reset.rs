use crate::req_res::AppError;
use crate::utils::{deserialize_from_messagepack, serialize_to_messagepack};
use base64::Engine;
use chrono::{DateTime, Utc};
use fred::prelude::*;
use log::error;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordResetReq {
    pub uuid: Uuid,
    pub expire: DateTime<Utc>,
    pub otp: String,
    pub reset_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PasswordResetResult {
    pub status: ResetStatus,
    pub reset_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ResetStatus {
    Valid,
    OtpInvalid,
    NotFound,
}

pub async fn new_password_reset_req(
    redis: &Client,
    session_uid: Uuid,
    otp: &str,
    expiry: DateTime<Utc>,
    uuid: Uuid,
) -> Result<(), AppError> {
    let reset_req = PasswordResetReq {
        uuid,
        expire: expiry,
        otp: otp.to_string(),
        reset_token: generate_reset_token(),
    };
    let packed = serialize_to_messagepack(&reset_req);
    let exp = Expiration::EX(660);
    redis
        .set(
            session_uid.to_string(),
            packed.as_slice(),
            Some(exp),
            None,
            false,
        )
        .await?;
    Ok(())
}

pub async fn verify_password_reset_otp(
    redis: &Client,
    session_uid: Uuid,
    otp: &str,
) -> Result<PasswordResetResult, AppError> {
    let bytes: Option<Vec<u8>> = redis.get(session_uid.to_string()).await?;
    match bytes {
        Some(data) => {
            let reset_req: PasswordResetReq = deserialize_from_messagepack(&data).map_err(|e| {
                error!("msgpack deserialization failure: {}", e.to_string());
                AppError::internal_error("unknown msg pack deserialization failure".to_string())
            })?;

            if reset_req.otp == otp {
                Ok(PasswordResetResult {
                    status: ResetStatus::Valid,
                    reset_token: Some(reset_req.reset_token),
                })
            } else {
                Ok(PasswordResetResult {
                    status: ResetStatus::OtpInvalid,
                    reset_token: None,
                })
            }
        }
        None => Ok(PasswordResetResult {
            status: ResetStatus::NotFound,
            reset_token: None,
        }),
    }
}

pub async fn verify_reset_token(
    redis: &Client,
    session_uid: Uuid,
    reset_token: &str,
) -> Result<Option<Uuid>, AppError> {
    let bytes: Option<Vec<u8>> = redis.get(session_uid.to_string()).await?;

    match bytes {
        Some(data) => {
            let reset_req: PasswordResetReq = deserialize_from_messagepack(&data).map_err(|e| {
                error!("msgpack deserialization failure: {}", e.to_string());
                AppError::internal_error("unknown msg pack deserialization failure".to_string())
            })?;

            if reset_req.reset_token == reset_token && reset_req.expire > Utc::now() {
                Ok(Some(reset_req.uuid))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn generate_reset_token() -> String {
    let mut rng = thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    base64::engine::general_purpose::URL_SAFE.encode(bytes)
}
