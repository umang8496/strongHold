//! ==============================================================================
//! Catalog Domain Service
//! ==============================================================================
//! Orchestrates business workflows, coordinates database transactions,
//! and wraps blocking Diesel operations inside `actix_web::web::block`.
//! ==============================================================================

use actix_web::web;
use common::{AppError, AppResult};
use database::{get_conn, DbPool};
use uuid::Uuid;

use crate::dto::{CreateProductRequest, ProductResponse, UpdateProductRequest};
use crate::models::{NewProduct, UpdateProduct};
use crate::repository;

/// Handles product creation workflow:
/// 1. Validates input invariants via the DTO.
/// 2. Offloads blocking DB work to Tokio's blocking thread pool via `web::block`.
/// 3. Returns the clean `ProductResponse` DTO.
pub async fn create_product(
    pool: web::Data<DbPool>,
    req: CreateProductRequest,
) -> AppResult<ProductResponse> {
    // Validate business constraints before borrowing any DB connection
    req.validate()?;

    web::block(move || {
        let mut conn = get_conn(&pool)?;

        // Map DTO -> Diesel Insertable Model
        let new_product = NewProduct {
            sku: &req.sku,
            title: &req.title,
            description: req.description.as_deref(),
            price_cents: req.price_cents,
            is_active: true,
        };

        // Persist via repository
        let created = repository::insert_product(&mut conn, &new_product)?;
        Ok(ProductResponse::from(created))
    })
    .await
    // Map Actix BlockingError (e.g. thread pool cancellation) to AppError
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}

/// Retrieves a single product by its UUID.
pub async fn get_product_by_id(
    pool: web::Data<DbPool>,
    product_id: Uuid,
) -> AppResult<ProductResponse> {
    web::block(move || {
        let mut conn = get_conn(&pool)?;
        let product = repository::find_product_by_id(&mut conn, product_id)?;
        Ok(ProductResponse::from(product))
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}

/// Lists products with pagination and optional price filters.
pub async fn list_products(
    pool: web::Data<DbPool>,
    page: i64,
    page_size: i64,
    min_price: Option<i64>,
    max_price: Option<i64>,
) -> AppResult<Vec<ProductResponse>> {
    web::block(move || {
        let mut conn = get_conn(&pool)?;
        let products = repository::list_products(
            &mut conn, page, page_size, min_price, max_price,
        )?;
        Ok(products.into_iter().map(ProductResponse::from).collect())
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}

/// Updates a product partially using PATCH semantics.
pub async fn update_product(
    pool: web::Data<DbPool>,
    product_id: Uuid,
    req: UpdateProductRequest,
) -> AppResult<ProductResponse> {
    req.validate()?;

    web::block(move || {
        let mut conn = get_conn(&pool)?;

        let changes = UpdateProduct {
            title: req.title.as_deref(),
            description: req.description.as_deref().map(Some),
            price_cents: req.price_cents,
            is_active: req.is_active,
            updated_at: Some(chrono::Utc::now()),
        };

        let updated = repository::update_product(&mut conn, product_id, &changes)?;
        Ok(ProductResponse::from(updated))
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}

/// Soft-deletes a product by marking it inactive.
pub async fn delete_product(
    pool: web::Data<DbPool>,
    product_id: Uuid,
) -> AppResult<()> {
    web::block(move || {
        let mut conn = get_conn(&pool)?;
        repository::soft_delete_product(&mut conn, product_id)
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}
