//! ==============================================================================
//! Orders Service & Cross-Domain Checkout Orchestration
//! ==============================================================================

use actix_web::web;
use common::{AppError, AppResult};
use database::{get_conn, DbPool};
use diesel::Connection;
use inventory::InventoryPort;
use std::sync::Arc;
use uuid::Uuid;

use crate::dto::{CreateOrderRequest, OrderDetailResponse};
use crate::models::{NewOrder, NewOrderItem, OrderStatus};
use crate::repository;

pub async fn checkout(
    pool: web::Data<DbPool>,
    inventory_port: web::Data<Arc<dyn InventoryPort>>,
    req: CreateOrderRequest,
) -> AppResult<OrderDetailResponse> {
    req.validate()?;

    web::block(move || {
        let mut conn = get_conn(&pool)?;

        // Run entire checkout within a single atomic database transaction
        conn.transaction::<OrderDetailResponse, AppError, _>(|tx_conn| {
            let mut line_items_to_insert = Vec::new();
            let mut total_cents: i64 = 0;
            let mut reserved_items: Vec<(Uuid, i32)> = Vec::new();

            // 1. Resolve product pricing and reserve inventory
            for item in &req.items {
                // Fetch product from catalog repository directly using our shared connection
                let product = catalog::repository::find_product_by_id(tx_conn, item.product_id)?;

                // Attempt to reserve stock via the port
                if let Err(err) = inventory_port.reserve_item(item.product_id, item.quantity) {
                    // Compensating action: release any items reserved earlier in this loop
                    for (prev_prod_id, prev_qty) in reserved_items {
                        let _ = inventory_port.release_item(prev_prod_id, prev_qty);
                    }
                    return Err(err);
                }
                reserved_items.push((item.product_id, item.quantity));

                let item_subtotal = product.price_cents * (item.quantity as i64);
                total_cents += item_subtotal;

                line_items_to_insert.push((item.product_id, item.quantity, product.price_cents));
            }

            // 2. Persist the Order Envelope
            let new_order = NewOrder {
                customer_id: req.customer_id,
                status: OrderStatus::Reserved.as_str(),
                total_amount_cents: total_cents,
            };
            let created_order = repository::insert_order(tx_conn, &new_order)?;

            // 3. Persist Order Items with snapshotted prices
            let items: Vec<NewOrderItem> = line_items_to_insert
                .into_iter()
                .map(|(prod_id, qty, price)| NewOrderItem {
                    order_id: created_order.id,
                    product_id: prod_id,
                    quantity: qty,
                    unit_price_cents: price,
                })
                .collect();

            let created_items = repository::insert_order_items(tx_conn, &items)?;

            Ok(OrderDetailResponse::from_parts(created_order, created_items))
        })
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}

pub async fn get_order_by_id(
    pool: web::Data<DbPool>,
    order_id: Uuid,
) -> AppResult<OrderDetailResponse> {
    web::block(move || {
        let mut conn = get_conn(&pool)?;
        let order = repository::find_order_by_id(&mut conn, order_id)?;
        let items = repository::find_items_by_order_id(&mut conn, order_id)?;
        Ok(OrderDetailResponse::from_parts(order, items))
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}

pub async fn cancel_order(
    pool: web::Data<DbPool>,
    inventory_port: web::Data<Arc<dyn InventoryPort>>,
    order_id: Uuid,
) -> AppResult<OrderDetailResponse> {
    web::block(move || {
        let mut conn = get_conn(&pool)?;

        conn.transaction::<OrderDetailResponse, AppError, _>(|tx_conn| {
            let order = repository::find_order_by_id(tx_conn, order_id)?;
            let current_status = OrderStatus::from_str(&order.status)?;

            if !current_status.can_transition_to(OrderStatus::Canceled) {
                return Err(AppError::BadRequest(format!(
                    "Cannot cancel order currently in status {}",
                    order.status
                )));
            }

            let items = repository::find_items_by_order_id(tx_conn, order_id)?;

            // If order had reserved stock, release it back to inventory
            if current_status == OrderStatus::Reserved {
                for item in &items {
                    inventory_port.release_item(item.product_id, item.quantity)?;
                }
            }

            let updated_order = repository::update_order_status(
                tx_conn,
                order_id,
                OrderStatus::Canceled.as_str(),
            )?;

            Ok(OrderDetailResponse::from_parts(updated_order, items))
        })
    })
    .await
    .map_err(|err| AppError::Internal(format!("Worker thread failed: {}", err)))?
}
