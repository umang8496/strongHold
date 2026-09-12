//! ==============================================================================
//! Inventory HTTP Controller
//! ==============================================================================

use actix_web::{get, patch, post, web, HttpResponse};
use common::AppResult;
use database::DbPool;
use uuid::Uuid;

use crate::dto::{AdjustStockRequest, InitializeStockRequest};
use crate::service;

#[post("/inventory")]
pub async fn initialize_stock_handler(
    pool: web::Data<DbPool>,
    payload: web::Json<InitializeStockRequest>,
) -> AppResult<HttpResponse> {
    let res = service::initialize_stock(pool, payload.into_inner()).await?;
    Ok(HttpResponse::Created().json(res))
}

#[get("/inventory/{product_id}")]
pub async fn get_stock_handler(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let product_id = path.into_inner();
    let res = service::get_stock(pool, product_id).await?;
    Ok(HttpResponse::Ok().json(res))
}

#[patch("/inventory/{product_id}/adjust")]
pub async fn adjust_stock_handler(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    payload: web::Json<AdjustStockRequest>,
) -> AppResult<HttpResponse> {
    let product_id = path.into_inner();
    let res = service::adjust_stock(pool, product_id, payload.into_inner()).await?;
    Ok(HttpResponse::Ok().json(res))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(initialize_stock_handler)
            .service(get_stock_handler)
            .service(adjust_stock_handler),
    );
}
