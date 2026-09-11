// @generated automatically by Diesel CLI.

pub mod catalog {
    diesel::table! {
        use diesel::sql_types::*;
        use diesel_full_text_search::*;

        catalog.products (id) {
            id -> Uuid,
            #[max_length = 64]
            sku -> Varchar,
            #[max_length = 255]
            title -> Varchar,
            description -> Nullable<Text>,
            price_cents -> Int8,
            is_active -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }
}
