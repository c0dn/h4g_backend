use crate::models::user::{AccountType, UserAddress};
use crate::req_res::{AppError, ClientErrorMessages, DataValidationError};
use crate::schema::private;
use diesel::AsChangeset;
use serde::Deserialize;

#[derive(Debug, AsChangeset)]
#[diesel(table_name = private::users)]
pub struct UpdateUser {
    pub resident_id: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub role: Option<AccountType>,
    pub dob: Option<String>,
    pub address: Option<serde_json::Value>,
    pub school: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PasswordChangeReq {
    pub password: String,
    pub confirm_password: String,
}

#[derive(Debug, Clone)]
pub struct PasswordChangeValidated {
    pub password: String,
    pub confirm_password: String,
}

impl TryInto<PasswordChangeValidated> for PasswordChangeReq {
    type Error = AppError;

    fn try_into(self) -> Result<PasswordChangeValidated, Self::Error> {
        let mut errors = vec![];

        if self.password != self.confirm_password {
            errors.push("Passwords do not match".to_string());
        }
        if self.password.len() < 10 {
            errors.push("Min password length 10".to_string());
        }

        if errors.is_empty() {
            Ok(PasswordChangeValidated {
                password: self.password,
                confirm_password: self.confirm_password,
            })
        } else {
            Err(AppError::bad_request::<ClientErrorMessages>(
                DataValidationError { errors }.into(),
            ))
        }
    }
}
