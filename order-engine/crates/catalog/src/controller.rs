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
use crate::dto::ProductResponse;

use common::error::{ErrorResponseBody};

/// Query parameters for GET /api/v1/products
#[derive(Debug, Deserialize)]
pub struct ProductQueryFilter {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
}

#[utoipa::path(
    post,
    path = "/api/v1/products",
    request_body = CreateProductRequest,
    responses(
        (status = 201, description = "Product created successfully", body = ProductResponse),
        (status = 400, description = "Validation error", body = ErrorResponseBody),
        (status = 409, description = "SKU already exists", body = ErrorResponseBody),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Catalog"
)]
#[post("/products")]
pub async fn create_product_handler(
    pool: web::Data<DbPool>,
    payload: web::Json<CreateProductRequest>,
) -> AppResult<HttpResponse> {
    let product = service::create_product(pool, payload.into_inner()).await?;
    Ok(HttpResponse::Created().json(product))
}

#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    params(
        ("id" = Uuid, Path, description = "Product unique identifier")
    ),
    responses(
        (status = 200, description = "Product found", body = ProductResponse),
        (status = 404, description = "Product not found", body = ErrorResponseBody),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Catalog"
)]
#[get("/products/{id}")]
pub async fn get_product_handler(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> AppResult<HttpResponse> {
    let product_id = path.into_inner();
    let product = service::get_product_by_id(pool, product_id).await?;
    Ok(HttpResponse::Ok().json(product))
}

#[utoipa::path(
    get,
    path = "/api/v1/products",
    responses(
        (status = 200, description = "List of all the products", body = ProductResponse),
        (status = 404, description = "Product not found", body = ErrorResponseBody),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Catalog"
)]
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

#[utoipa::path(
    patch,
    path = "/api/v1/products/{id}",
    responses(
        (status = 200, description = "Updates the given product", body = ProductResponse),
        (status = 404, description = "Product not found", body = ErrorResponseBody),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Catalog"
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/products/{id}",
    responses(
        (status = 204, description = "Deletes the given product", body = ProductResponse),
        (status = 404, description = "Product not found", body = ErrorResponseBody),
        (status = 500, description = "Internal server error", body = ErrorResponseBody)
    ),
    tag = "Catalog"
)]
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
