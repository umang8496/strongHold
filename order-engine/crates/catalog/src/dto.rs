//! ==============================================================================
//! HTTP Data Transfer Objects (DTOs) for Catalog
//! ==============================================================================
//! Decouples external API contracts from internal database representations.
//! Handles incoming request body validation and outgoing JSON responses.
//! ==============================================================================

use chrono::{DateTime, Utc};
use common::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Product;

/// Payload received when creating a new product via POST /api/v1/products
#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub sku: String,
    pub title: String,
    pub description: Option<String>,
    pub price_cents: i64,
}

impl CreateProductRequest {
    /// Invariant validation: ensure data integrity before calling the repository.
    pub fn validate(&self) -> AppResult<()> {
        if self.sku.trim().is_empty() {
            return Err(AppError::BadRequest("SKU cannot be empty".to_string()));
        }
        if self.title.trim().is_empty() {
            return Err(AppError::BadRequest("Title cannot be empty".to_string()));
        }
        if self.price_cents < 0 {
            return Err(AppError::BadRequest(
                "Price in cents must be non-negative".to_string(),
            ));
        }
        Ok(())
    }
}

/// Payload received when updating a product via PATCH /api/v1/products/{id}
#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub price_cents: Option<i64>,
    pub is_active: Option<bool>,
}

impl UpdateProductRequest {
    pub fn validate(&self) -> AppResult<()> {
        if let Some(ref title) = self.title {
            if title.trim().is_empty() {
                return Err(AppError::BadRequest("Title cannot be empty".to_string()));
            }
        }
        if let Some(price) = self.price_cents {
            if price < 0 {
                return Err(AppError::BadRequest(
                    "Price in cents must be non-negative".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Standardized JSON response returned to clients representing a product.
#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub sku: String,
    pub title: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Convert an internal database entity into an external HTTP response DTO.
impl From<Product> for ProductResponse {
    fn from(p: Product) -> Self {
        Self {
            id: p.id,
            sku: p.sku,
            title: p.title,
            description: p.description,
            price_cents: p.price_cents,
            is_active: p.is_active,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}
