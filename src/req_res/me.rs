use crate::helper::hash_password_phone;
use crate::schema::private;
use diesel::{AsChangeset, Insertable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::user::{AccountType, User};
use crate::paseto::{generate_access_token, generate_refresh_token};
use crate::regex;
use crate::req_res::{AppError, ClientErrorMessages, DataValidationError};


#[derive(Debug, AsChangeset)]
#[diesel(table_name = private::users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub role: Option<AccountType>,
}
