// @generated automatically by Diesel CLI.

pub mod private {
    pub mod sql_types {
        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "account_type", schema = "private"))]
        pub struct AccountType;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "transaction_type", schema = "private"))]
        pub struct TransactionType;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use diesel_full_text_search::Tsvector;

        private.products (uuid) {
            uuid -> Uuid,
            title -> Text,
            image_path -> Text,
            description -> Text,
            stock -> Int4,
            cost -> Int4,
            search_vector -> Tsvector,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use diesel_full_text_search::Tsvector;
        use super::sql_types::TransactionType;

        private.transactions (id) {
            id -> Int4,
            wallet_id -> Int4,
            amount -> Int4,
            transaction_type -> TransactionType,
            #[max_length = 255]
            description -> Varchar,
            created_at -> Timestamp,
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
            school -> Nullable<Text>,
            force_pw_change -> Bool,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use diesel_full_text_search::Tsvector;

        private.wallets (id) {
            id -> Int4,
            user_uuid -> Uuid,
            balance -> Int4,
            updated_at -> Timestamp,
        }
    }

    diesel::joinable!(transactions -> wallets (wallet_id));
    diesel::joinable!(wallets -> users (user_uuid));

    diesel::allow_tables_to_appear_in_same_query!(products, transactions, users, wallets,);
}
