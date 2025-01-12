use crate::helper::hash_password_phone;
use crate::models::user::AccountType;
use crate::regex;
use crate::req_res::auth::NewUser;
use crate::req_res::me::UpdateUser;
use crate::req_res::{AppError, ClientErrorMessages, DataValidationError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AdminNewUserReq {
    pub username: String,
    pub email: String,
    pub name: String,
    pub phone: String,
    pub role: AccountType,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminUpdateUserReq {
    pub username: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub role: Option<AccountType>,
}

impl TryInto<NewUser> for AdminNewUserReq {
    type Error = AppError;

    fn try_into(self) -> Result<NewUser, Self::Error> {
        let mut errors = vec![];
        let re = regex!(r"^[a-zA-Z0-9_]+$");
        if self.username.len() < 4 {
            errors.push("Username too short".to_string());
        }
        if !re.is_match(&self.username) {
            errors.push("Username can only contain numbers, letters, and underscores".to_string());
        }
        if self.phone.len() != 8 {
            errors.push("Invalid Singapore phone number".to_string());
        }
        if errors.is_empty() {
            Ok(NewUser {
                username: self.username,
                email: self.email,
                name: self.name,
                password: hash_password_phone("placeholder")?,
                phone: hash_password_phone(&self.phone)?,
                role: self.role,
                active: true,
            })
        } else {
            Err(AppError::bad_request::<ClientErrorMessages>(
                DataValidationError { errors }.into(),
            ))
        }
    }
}

impl TryInto<UpdateUser> for AdminUpdateUserReq {
    type Error = AppError;

    fn try_into(self) -> Result<UpdateUser, Self::Error> {
        let mut errors = vec![];

        if let Some(username) = &self.username {
            let re = regex!(r"^[a-zA-Z0-9_]+$");
            if username.len() < 4 {
                errors.push("Username too short".to_string());
            }
            if !re.is_match(username) {
                errors.push(
                    "Username can only contain numbers, letters, and underscores".to_string(),
                );
            }
        }

        if let Some(phone) = &self.phone {
            if phone.len() != 8 {
                errors.push("Invalid Singapore phone number".to_string());
            }
        }

        if errors.is_empty() {
            Ok(UpdateUser {
                username: self.username,
                email: self.email,
                name: self.name,
                phone: self.phone.map(|p| hash_password_phone(&p)).transpose()?,
                role: self.role,
            })
        } else {
            Err(AppError::bad_request::<ClientErrorMessages>(
                DataValidationError { errors }.into(),
            ))
        }
    }
}
