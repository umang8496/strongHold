//! ==============================================================================
//! Diesel Database Models for Inventory
//! ==============================================================================

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use database::schema::inventory::stocks;

/// Represents a row read from `inventory.stocks`.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = stocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Stock {
    pub id: Uuid,
    pub product_id: Uuid,
    pub warehouse_code: String,
    pub available_quantity: i32,
    pub reserved_quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents the payload to initialize stock for a product.
#[derive(Debug, Insertable)]
#[diesel(table_name = stocks)]
pub struct NewStock<'a> {
    pub product_id: Uuid,
    pub warehouse_code: &'a str,
    pub available_quantity: i32,
    pub reserved_quantity: i32,
}
