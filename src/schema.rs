// @generated automatically by Diesel CLI.

pub mod private {
    pub mod sql_types {
        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "account_type", schema = "private"))]
        pub struct AccountType;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use diesel_full_text_search::Tsvector;

        private.products (uuid) {
            uuid -> Uuid,
            title -> Text,
            description -> Text,
            stock -> Int4,
            cost -> Int4,
            search_vector -> Tsvector,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use diesel_full_text_search::Tsvector;
        use super::sql_types::AccountType;

        private.users (uuid) {
            uuid -> Uuid,
            resident_id -> Text,
            name -> Text,
            phone -> Text,
            password -> Text,
            email -> Text,
            role -> AccountType,
            active -> Bool,
            dob -> Nullable<Text>,
            address -> Nullable<Jsonb>,
            force_pw_change -> Bool,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(products, users,);
}
