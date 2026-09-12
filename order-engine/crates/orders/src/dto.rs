//! ==============================================================================
//! HTTP Data Transfer Objects for Orders
//! ==============================================================================

use chrono::{DateTime, Utc};
use common::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Order, OrderItem};

#[derive(Debug, Deserialize)]
pub struct CreateOrderItemRequest {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: Uuid,
    pub items: Vec<CreateOrderItemRequest>,
}

impl CreateOrderRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.items.is_empty() {
            return Err(AppError::BadRequest(
                "Order must contain at least one item".to_string(),
            ));
        }
        for item in &self.items {
            if item.quantity <= 0 {
                return Err(AppError::BadRequest(
                    "Item quantity must be strictly greater than zero".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct OrderItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price_cents: i64,
}

impl From<OrderItem> for OrderItemResponse {
    fn from(item: OrderItem) -> Self {
        Self {
            id: item.id,
            product_id: item.product_id,
            quantity: item.quantity,
            unit_price_cents: item.unit_price_cents,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OrderDetailResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub status: String,
    pub total_amount_cents: i64,
    pub items: Vec<OrderItemResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OrderDetailResponse {
    pub fn from_parts(order: Order, items: Vec<OrderItem>) -> Self {
        Self {
            id: order.id,
            customer_id: order.customer_id,
            status: order.status,
            total_amount_cents: order.total_amount_cents,
            items: items.into_iter().map(OrderItemResponse::from).collect(),
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}
