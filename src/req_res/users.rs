use crate::helper::hash_password;
use crate::models::user::{AccountType, UserAddress};
use crate::req_res::auth::NewUser;
use crate::req_res::me::UpdateUser;
use crate::req_res::{AppError, ClientErrorMessages, DataValidationError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AdminNewUserReq {
    pub resident_id: String,
    pub email: String,
    pub name: String,
    pub phone: String,
    pub role: AccountType,
    pub address: Option<UserAddress>,
    pub dob: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminUpdateUserReq {
    pub resident_id: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub role: Option<AccountType>,
    pub address: Option<UserAddress>,
    pub dob: Option<String>,
}

impl TryInto<NewUser> for AdminNewUserReq {
    type Error = AppError;

    fn try_into(self) -> Result<NewUser, Self::Error> {
        let mut errors = vec![];
        if self.phone.len() != 8 {
            errors.push("Invalid Singapore phone number".to_string());
        }
        if errors.is_empty() {
            Ok(NewUser {
                resident_id: self.resident_id,
                email: self.email,
                name: self.name,
                password: hash_password("placeholder")?,
                phone: self.phone,
                role: self.role,
                active: true,
                dob: self.dob,
                address: self.address.map(|a| serde_json::to_value(&a).unwrap()),
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

        if let Some(phone) = &self.phone {
            if phone.len() != 8 {
                errors.push("Invalid Singapore phone number".to_string());
            }
        }

        if errors.is_empty() {
            Ok(UpdateUser {
                resident_id: self.resident_id,
                email: self.email,
                name: self.name,
                phone: self.phone,
                role: self.role,
                dob: self.dob,
                address: self.address.map(|a| serde_json::to_value(&a).unwrap()),
            })
        } else {
            Err(AppError::bad_request::<ClientErrorMessages>(
                DataValidationError { errors }.into(),
            ))
        }
    }
}
