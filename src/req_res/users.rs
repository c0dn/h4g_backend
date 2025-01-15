use crate::helper::hash_password;
use crate::models::user::{AccountType, User, UserAddress};
use crate::models::wallet::Wallet;
use crate::req_res::auth::NewUser;
use crate::req_res::me::UpdateUser;
use crate::req_res::{AppError, ClientErrorMessages, DataValidationError};
use num_traits::cast::ToPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
pub struct AdminNewUserReq {
    pub resident_id: String,
    pub email: String,
    pub name: String,
    pub phone: String,
    pub role: AccountType,
    pub address: Option<UserAddress>,
    pub dob: Option<String>,
    pub school: Option<String>,
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
    pub school: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DetailedUser {
    pub uuid: Uuid,
    pub resident_id: String,
    pub email: String,
    pub name: String,
    pub phone: String,
    pub balance: i32,
    pub dob: String,
    pub school: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct DetailedUserFull {
    pub uuid: Uuid,
    pub resident_id: String,
    pub email: String,
    pub name: String,
    pub phone: String,
    pub balance: i32,
    pub dob: String,
    pub school: String,
    pub address: Option<UserAddress>,
    pub role: AccountType,
}

impl From<(User, Option<Wallet>)> for DetailedUser {
    fn from((user, wallet): (User, Option<Wallet>)) -> DetailedUser {
        DetailedUser {
            uuid: user.uuid,
            resident_id: user.resident_id,
            email: user.email,
            name: user.name,
            phone: user.phone,
            balance: wallet.map(|w| w.balance.to_i32().unwrap_or(0)).unwrap_or(0),
            dob: user.dob.unwrap_or("Not specified".to_string()),
            school: user.school.unwrap_or("Not schooling".to_string()),
        }
    }
}

impl From<(User, Option<Wallet>)> for DetailedUserFull {
    fn from((user, wallet): (User, Option<Wallet>)) -> DetailedUserFull {
        let address_value = user
            .address
            .and_then(|addr| serde_json::from_value::<UserAddress>(addr).ok());

        DetailedUserFull {
            uuid: user.uuid,
            resident_id: user.resident_id,
            email: user.email,
            name: user.name,
            phone: user.phone,
            balance: wallet.map(|w| w.balance.to_i32().unwrap_or(0)).unwrap_or(0),
            dob: user.dob.unwrap_or("Not specified".to_string()),
            school: user.school.unwrap_or("Not schooling".to_string()),
            address: address_value,
            role: user.role,
        }
    }
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
                school: self.school,
                force_pw_change: true,
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
                school: self.school,
                address: self.address.map(|a| serde_json::to_value(&a).unwrap()),
            })
        } else {
            Err(AppError::bad_request::<ClientErrorMessages>(
                DataValidationError { errors }.into(),
            ))
        }
    }
}
