use crate::helper::hash_password;
use crate::schema::private;
use diesel::Insertable;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::user::{AccountType, User};
use crate::paseto::{generate_access_token, generate_refresh_token};
use crate::regex;
use crate::req_res::{AppError, ClientErrorMessages, DataValidationError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppInitRequest {
    pub username: String,
    pub email: String,
    pub name: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Debug, Deserialize, Insertable, Clone)]
#[diesel(table_name = private::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: AccountType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedactedUser {
    pub uuid: String,
    pub username: String,
    pub name: String,
    pub email: String,
    pub role: AccountType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAuthenticationResponse {
    pub user: RedactedUser,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewTokens {
    pub access_token: String,
    pub refresh_token: String,
}

impl NewTokens {
    pub(crate) fn new(uuid: Uuid, role: AccountType) -> Self {
        let access_token = generate_access_token(&uuid.to_string(), format!("{:?}", role).as_str());
        let refresh_token =
            generate_refresh_token(&uuid.to_string(), format!("{:?}", role).as_str());
        NewTokens {
            access_token,
            refresh_token,
        }
    }
}

impl Into<UserAuthenticationResponse> for User {
    fn into(self) -> UserAuthenticationResponse {
        let redacted = RedactedUser {
            uuid: self.uuid.to_string(),
            username: self.username.clone(),
            name: self.name.clone(),
            email: self.email.clone(),
            role: self.role,
        };
        let access_token =
            generate_access_token(&self.uuid.to_string(), format!("{:?}", self.role).as_str());
        let refresh_token =
            generate_refresh_token(&self.uuid.to_string(), format!("{:?}", self.role).as_str());
        UserAuthenticationResponse {
            user: redacted,
            access_token,
            refresh_token,
        }
    }
}

impl TryInto<NewUser> for AppInitRequest {
    type Error = AppError;

    fn try_into(self) -> Result<NewUser, Self::Error> {
        let mut errors = vec![];
        let role: AccountType = AccountType::SuperAdmin;
        let re = regex!(r"^[a-zA-Z0-9_]+$");
        if self.username.len() < 4 {
            errors.push("Username too short".to_string());
        }
        if !re.is_match(&self.username) {
            errors.push("Username can only contain numbers, letters, and underscores".to_string());
        }
        if self.password != self.confirm_password {
            errors.push("Passwords do not match".to_string());
        }
        if self.password.len() < 10 {
            errors.push("Min password length 10".to_string());
        }
        if errors.is_empty() {
            Ok(NewUser {
                username: self.username,
                email: self.email,
                name: self.name,
                password: hash_password(&self.password)?,
                role,
            })
        } else {
            Err(AppError::bad_request::<ClientErrorMessages>(
                DataValidationError { errors }.into(),
            ))
        }
    }
}
