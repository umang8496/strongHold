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
// @generated automatically by Diesel CLI.

pub mod orders {
    diesel::table! {
        use diesel::sql_types::*;

        orders.order_items (id) {
            id -> Uuid,
            order_id -> Uuid,
            product_id -> Uuid,
            quantity -> Int4,
            unit_price_cents -> Int8,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;

        orders.orders (id) {
            id -> Uuid,
            customer_id -> Uuid,
            #[max_length = 32]
            status -> Varchar,
            total_amount_cents -> Int8,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::joinable!(order_items -> orders (order_id));

    diesel::allow_tables_to_appear_in_same_query!(order_items, orders,);
}
