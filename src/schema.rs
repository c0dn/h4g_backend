// @generated automatically by Diesel CLI.

pub mod private {
    pub mod sql_types {
        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "account_type", schema = "private"))]
        pub struct AccountType;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::AccountType;

        private.users (uuid) {
            uuid -> Uuid,
            username -> Text,
            name -> Text,
            phone -> Text,
            password -> Text,
            email -> Text,
            role -> AccountType,
            active -> Bool,
        }
    }
}
