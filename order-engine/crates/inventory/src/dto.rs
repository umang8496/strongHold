//! ==============================================================================
//! HTTP Data Transfer Objects & Domain Payloads for Inventory
//! ==============================================================================

use chrono::{DateTime, Utc};
use common::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Stock;

/// Payload received when initializing or resetting stock
#[derive(Debug, Deserialize)]
pub struct InitializeStockRequest {
    pub product_id: Uuid,
    pub warehouse_code: Option<String>,
    pub quantity: i32,
}

impl InitializeStockRequest {
    pub fn validate(&self) -> AppResult<()> {
        if self.quantity < 0 {
            return Err(AppError::BadRequest(
                "Initial stock quantity cannot be negative".to_string(),
            ));
        }
        Ok(())
    }
}

/// Payload received to manually adjust available stock (restock or audit deduction)
#[derive(Debug, Deserialize)]
pub struct AdjustStockRequest {
    pub delta: i32, // Positive adds stock; negative removes available stock
}

/// Outgoing stock representation
#[derive(Debug, Serialize)]
pub struct StockResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub warehouse_code: String,
    pub available_quantity: i32,
    pub reserved_quantity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Stock> for StockResponse {
    fn from(s: Stock) -> Self {
        Self {
            id: s.id,
            product_id: s.product_id,
            warehouse_code: s.warehouse_code,
            available_quantity: s.available_quantity,
            reserved_quantity: s.reserved_quantity,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}
