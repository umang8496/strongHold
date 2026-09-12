// @generated automatically by Diesel CLI.

pub mod catalog {
    diesel::table! {
        use diesel::sql_types::*;

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
// @generated automatically by Diesel CLI.

pub mod inventory {
    diesel::table! {
        use diesel::sql_types::*;

        inventory.stocks (id) {
            id -> Uuid,
            product_id -> Uuid,
            #[max_length = 32]
            warehouse_code -> Varchar,
            available_quantity -> Int4,
            reserved_quantity -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }
}
