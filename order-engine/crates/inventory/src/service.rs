//! ==============================================================================
//! Inventory Service & Public Port Interface
//! ==============================================================================


use actix_web::web;
use common::{AppError, AppResult};
use database::{get_conn, DbPool};
use uuid::Uuid;

use crate::dto::{AdjustStockRequest, InitializeStockRequest, StockResponse};
use crate::models::NewStock;
use crate::repository;

/// The public trait that other domains (like `orders`) use to talk to Inventory.
/// Today, this calls local DB queries in-process. Tomorrow, it can be swapped
/// for an HTTP or gRPC client without changing the caller's code.
pub trait InventoryPort: Send + Sync {
    fn reserve_item(&self, product_id: Uuid, quantity: i32) -> AppResult<()>;
    fn release_item(&self, product_id: Uuid, quantity: i32) -> AppResult<()>;
}

/// Concrete in-memory implementation of InventoryPort backed by the local DB pool
#[derive(Clone)]
pub struct LocalInventoryService {
    pool: DbPool,
}

impl LocalInventoryService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl InventoryPort for LocalInventoryService {
    fn reserve_item(&self, prod_id: Uuid, quantity: i32) -> AppResult<()> {
        let mut conn = get_conn(&self.pool)?;
        repository::reserve_stock_atomic(&mut conn, prod_id, quantity)?;
        Ok(())
    }

    fn release_item(&self, prod_id: Uuid, quantity: i32) -> AppResult<()> {
        let mut conn = get_conn(&self.pool)?;
        repository::release_reservation_atomic(&mut conn, prod_id, quantity)?;
        Ok(())
    }
}

/// HTTP workflow to initialize stock
pub async fn initialize_stock(
    pool: web::Data<DbPool>,
    req: InitializeStockRequest,
) -> AppResult<StockResponse> {
    req.validate()?;

    web::block(move || {
        let mut conn = get_conn(&pool)?;

        let warehouse = req.warehouse_code.as_deref().unwrap_or("WH-DEFAULT");
        let new_stock = NewStock {
            product_id: req.product_id,
            warehouse_code: warehouse,
            available_quantity: req.quantity,
            reserved_quantity: 0,
        };

        let created = repository::insert_stock(&mut conn, &new_stock)?;
        Ok(StockResponse::from(created))
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}

/// HTTP workflow to adjust available stock
pub async fn adjust_stock(
    pool: web::Data<DbPool>,
    product_id: Uuid,
    req: AdjustStockRequest,
) -> AppResult<StockResponse> {
    web::block(move || {
        let mut conn = get_conn(&pool)?;
        let updated = repository::adjust_available_stock(&mut conn, product_id, req.delta)?;
        Ok(StockResponse::from(updated))
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}

/// HTTP workflow to view stock by product_id
pub async fn get_stock(
    pool: web::Data<DbPool>,
    product_id: Uuid,
) -> AppResult<StockResponse> {
    web::block(move || {
        let mut conn = get_conn(&pool)?;
        let stock = repository::find_by_product_id(&mut conn, product_id)?;
        Ok(StockResponse::from(stock))
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}
