//! ==============================================================================
//! Orders HTTP Controller
//! ==============================================================================

use actix_web::{get, post, web, HttpResponse};
use common::AppResult;
use database::DbPool;
use inventory::InventoryPort;
use std::sync::Arc;
use uuid::Uuid;

use crate::dto::CreateOrderRequest;
use crate::service;

#[post("/orders")]
pub async fn create_order_handler(
    pool: web::Data<DbPool>,
    inventory_port: web::Data<Arc<dyn InventoryPort>>,
    payload: web::Json<CreateOrderRequest>,
) -> AppResult<HttpResponse> {
    let order = service::checkout(pool, inventory_port, payload.into_inner()).await?;
    Ok(HttpResponse::Created().json(order))
}

#[get("/orders/{id}")]
pub async fn get_order_handler(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let order_id = path.into_inner();
    let order = service::get_order_by_id(pool, order_id).await?;
    Ok(HttpResponse::Ok().json(order))
}

#[post("/orders/{id}/cancel")]
pub async fn cancel_order_handler(
    pool: web::Data<DbPool>,
    inventory_port: web::Data<Arc<dyn InventoryPort>>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let order_id = path.into_inner();
    let order = service::cancel_order(pool, inventory_port, order_id).await?;
    Ok(HttpResponse::Ok().json(order))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(create_order_handler)
            .service(get_order_handler)
            .service(cancel_order_handler),
    );
}
