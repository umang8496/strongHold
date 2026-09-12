//! ==============================================================================
//! Diesel Database Models & State Machine for Orders
//! ==============================================================================

use chrono::{DateTime, Utc};
use common::{AppError, AppResult};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use database::schema::orders::{order_items, orders};

/// Discrete states of an order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Draft,
    Reserved,
    Paid,
    Completed,
    Canceled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Draft => "DRAFT",
            OrderStatus::Reserved => "RESERVED",
            OrderStatus::Paid => "PAID",
            OrderStatus::Completed => "COMPLETED",
            OrderStatus::Canceled => "CANCELED",
        }
    }

    pub fn from_str(s: &str) -> AppResult<Self> {
        match s {
            "DRAFT" => Ok(OrderStatus::Draft),
            "RESERVED" => Ok(OrderStatus::Reserved),
            "PAID" => Ok(OrderStatus::Paid),
            "COMPLETED" => Ok(OrderStatus::Completed),
            "CANCELED" => Ok(OrderStatus::Canceled),
            unknown => Err(AppError::BadRequest(format!("Unknown order status: {}", unknown))),
        }
    }

    /// Enforces state transition invariants
    pub fn can_transition_to(&self, next: OrderStatus) -> bool {
        match (self, next) {
            (OrderStatus::Draft, OrderStatus::Reserved) => true,
            (OrderStatus::Reserved, OrderStatus::Paid) => true,
            (OrderStatus::Reserved, OrderStatus::Canceled) => true,
            (OrderStatus::Paid, OrderStatus::Completed) => true,
            (OrderStatus::Paid, OrderStatus::Canceled) => true,
            _ => false,
        }
    }
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Order envelope row from `orders.orders`
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub status: String,
    pub total_amount_cents: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Order line item row from `orders.order_items`
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Associations, Serialize)]
#[diesel(belongs_to(Order))]
#[diesel(table_name = order_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = orders)]
pub struct NewOrder<'a> {
    pub customer_id: Uuid,
    pub status: &'a str,
    pub total_amount_cents: i64,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = order_items)]
pub struct NewOrderItem {
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price_cents: i64,
}
