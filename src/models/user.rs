use crate::schema::private;
use diesel::{Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "private::sql_types::AccountType"]
pub enum AccountType {
    #[default]
    User,
    Admin,
    SuperAdmin,
}

impl TryFrom<&str> for AccountType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Admin" => Ok(AccountType::Admin),
            "User" => Ok(AccountType::User),
            "SuperAdmin" => Ok(AccountType::SuperAdmin),
            _ => Err(format!("Unknown role: {}", value)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAddress {
    pub bunk: String,
    pub floor: u8,
    pub unit: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = private::users)]
pub struct User {
    pub uuid: Uuid,
    pub resident_id: String,
    pub name: String,
    pub phone: String,
    pub password: String,
    pub email: String,
    pub role: AccountType,
    pub active: bool,
    pub dob: Option<String>,
    pub address: Option<serde_json::Value>,
    pub force_pw_change: bool,
}
