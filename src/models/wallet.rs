use crate::schema::private;
use chrono::NaiveDateTime;
use diesel::{Queryable, Selectable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = private::wallets)]
pub struct Wallet {
    pub id: i32,
    pub user_uuid: Uuid,
    pub balance: i32,
    pub updated_at: NaiveDateTime,
}
#[derive(Debug, Serialize, Deserialize, Default, Copy, Clone, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "private::sql_types::TransactionType"]
pub enum TransactionType {
    #[default]
    Debit,
    Credit,
}

impl TryFrom<&str> for TransactionType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Debit" => Ok(TransactionType::Debit),
            "Credit" => Ok(TransactionType::Credit),
            _ => Err(format!("Unknown transaction type: {}", value)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Selectable)]
#[diesel(table_name = private::transactions)]
pub struct Transaction {
    pub id: i32,
    pub wallet_id: i32,
    pub amount: i32,
    pub transaction_type: TransactionType,
    pub description: String,
    pub created_at: NaiveDateTime,
}
