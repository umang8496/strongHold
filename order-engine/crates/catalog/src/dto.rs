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
use utoipa::ToSchema;

use crate::models::Product;

/// Payload received when creating a new product via POST /api/v1/products
#[derive(Debug, Deserialize, ToSchema)]
#[schema(as = catalog::CreateProductRequest)]
pub struct CreateProductRequest {
    #[schema(example = "KB-MECH-01")]
    pub sku: String,

    #[schema(example = "Custom Mechanical Keyboard")]
    pub title: String,

    #[schema(example = "Hot-swappable tactile switch keyboard")]
    pub description: Option<String>,

    #[schema(example = 12900)]
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
#[derive(Debug, Deserialize, ToSchema)]
#[schema(as = catalog::UpdateProductRequest)]
pub struct UpdateProductRequest {
    /// Title for the product that to be updated
    #[schema(example = "Custom Mechanical Keyboard V2")]
    pub title: Option<String>,

    /// Description for the product that to be updated
    #[schema(example = "Hot-swappable mechanical keyboard with tactile switches")]
    pub description: Option<String>,

    /// Price for the product that to be updated
    #[schema(example = 13900)]
    pub price_cents: Option<i64>,

    /// Active status for the product that to be updated
    #[schema(example = "True or False")]
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
/// Represents a sellable physical product in the catalog.
/// Markdown written in docstrings is automatically transferred to OpenAPI descriptions.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = catalog::ProductResponse)]
pub struct ProductResponse {
    /// Universally unique identifier of the catalog item.
    #[schema(example = "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8")]
    pub id: Uuid,

    /// Unique Stock Keeping Unit (alphanumeric identifier).
    #[schema(example = "KB-MECH-01", max_length = 64, min_length = 3)]
    pub sku: String,

    /// Display title for retail presentation.
    #[schema(example = "Mechanical Keyboard V2", max_length = 255)]
    pub title: String,

    /// Extended markdown-compatible product description.
    #[schema(example = "Hot-swappable mechanical keyboard featuring tactile switches.")]
    pub description: Option<String>,

    /// Retail cost stored in the smallest fractional unit (cents) to avoid precision loss.
    #[schema(example = 12900, minimum = 0)]
    pub price_cents: i64,

    /// Flag designating whether the product is queryable by standard customers.
    #[schema(default = true)]
    pub is_active: bool,

    /// Creation timestamp in RFC 3339 format.
    #[schema(value_type = String, format = DateTime, example = "2026-03-31T08:30:00Z")]
    pub created_at: DateTime<Utc>,

    /// Update timestamp in RFC 3339 format.
    #[schema(value_type = String, format = DateTime, example = "2026-03-31T08:30:00Z")]
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
