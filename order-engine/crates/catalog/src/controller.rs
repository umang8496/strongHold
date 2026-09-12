//! ==============================================================================
//! Catalog HTTP Controller
//! ==============================================================================
//! Defines Actix route handlers, parameter extraction (JSON bodies, path params,
//! query params), and configures the domain's route registry.
//! ==============================================================================

use actix_web::{delete, get, patch, post, web, HttpResponse};
use common::AppResult;
use database::DbPool;
use serde::Deserialize;
use uuid::Uuid;

use crate::dto::{CreateProductRequest, UpdateProductRequest};
use crate::service;

/// Query parameters for GET /api/v1/products
#[derive(Debug, Deserialize)]
pub struct ProductQueryFilter {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
}

#[post("/products")]
pub async fn create_product_handler(
    pool: web::Data<DbPool>,
    payload: web::Json<CreateProductRequest>,
) -> AppResult<HttpResponse> {
    let product = service::create_product(pool, payload.into_inner()).await?;
    Ok(HttpResponse::Created().json(product))
}

#[get("/products/{id}")]
pub async fn get_product_handler(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let product_id = path.into_inner();
    let product = service::get_product_by_id(pool, product_id).await?;
    Ok(HttpResponse::Ok().json(product))
}

#[get("/products")]
pub async fn list_products_handler(
    pool: web::Data<DbPool>,
    query: web::Query<ProductQueryFilter>,
) -> AppResult<HttpResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);

    let products = service::list_products(
        pool,
        page,
        page_size,
        query.min_price,
        query.max_price,
    )
    .await?;

    Ok(HttpResponse::Ok().json(products))
}

#[patch("/products/{id}")]
pub async fn update_product_handler(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateProductRequest>,
) -> AppResult<HttpResponse> {
    let product_id = path.into_inner();
    let updated = service::update_product(pool, product_id, payload.into_inner()).await?;
    Ok(HttpResponse::Ok().json(updated))
}

#[delete("/products/{id}")]
pub async fn delete_product_handler(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let product_id = path.into_inner();
    service::delete_product(pool, product_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Registers all catalog routes onto an Actix service config scope.
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(create_product_handler)
            .service(get_product_handler)
            .service(list_products_handler)
            .service(update_product_handler)
            .service(delete_product_handler),
    );
}
