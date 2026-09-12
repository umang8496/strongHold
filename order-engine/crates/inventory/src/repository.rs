//! ==============================================================================
//! Inventory Repository with Concurrency Safeguards
//! ==============================================================================

use common::{AppError, AppResult};
use database::map_diesel_error;
use database::schema::inventory::stocks::dsl::*;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::{NewStock, Stock};

/// Inserts a stock record for a product.
pub fn insert_stock(conn: &mut PgConnection, new_stock: &NewStock) -> AppResult<Stock> {
    diesel::insert_into(stocks)
        .values(new_stock)
        .get_result::<Stock>(conn)
        .map_err(|err| map_diesel_error(err, "Stock"))
}

/// Finds stock by product_id.
pub fn find_by_product_id(conn: &mut PgConnection, prod_id: Uuid) -> AppResult<Stock> {
    stocks
        .filter(product_id.eq(prod_id))
        .first::<Stock>(conn)
        .map_err(|err| map_diesel_error(err, "Stock"))
}

/// Atomically adjusts available inventory (used for replenishments or manual deductions).
pub fn adjust_available_stock(
    conn: &mut PgConnection,
    prod_id: Uuid,
    qty_delta: i32,
) -> AppResult<Stock> {
    let result = diesel::update(stocks.filter(product_id.eq(prod_id)))
        .set((
            available_quantity.eq(available_quantity + qty_delta),
            updated_at.eq(chrono::Utc::now()),
        ))
        .get_result::<Stock>(conn)
        .map_err(|err| map_diesel_error(err, "Stock"))?;

    Ok(result)
}

/// ATOMIC RESERVATION (Prevents Overselling Race Conditions)
///
/// Moves quantity from `available_quantity` to `reserved_quantity` in a single SQL operation.
/// The `filter(available_quantity.ge(qty))` clause guarantees that even under extreme
/// concurrent requests, PostgreSQL locks the row and only succeeds if there is sufficient stock.
pub fn reserve_stock_atomic(
    conn: &mut PgConnection,
    prod_id: Uuid,
    qty: i32,
) -> AppResult<Stock> {
    let affected = diesel::update(
        stocks
            .filter(product_id.eq(prod_id))
            .filter(available_quantity.ge(qty)), // Atomic invariant guard
    )
    .set((
        available_quantity.eq(available_quantity - qty),
        reserved_quantity.eq(reserved_quantity + qty),
        updated_at.eq(chrono::Utc::now()),
    ))
    .get_result::<Stock>(conn);

    match affected {
        Ok(stock) => Ok(stock),
        Err(diesel::result::Error::NotFound) => {
            // Either the product does not exist, or available_quantity was less than qty
            Err(AppError::Conflict(format!(
                "Insufficient available stock to reserve {} units for product {}",
                qty, prod_id
            )))
        }
        Err(err) => Err(map_diesel_error(err, "Stock")),
    }
}

/// Releases a prior reservation back to available stock (e.g. order canceled or timed out).
pub fn release_reservation_atomic(
    conn: &mut PgConnection,
    prod_id: Uuid,
    qty: i32,
) -> AppResult<Stock> {
    let affected = diesel::update(
        stocks
            .filter(product_id.eq(prod_id))
            .filter(reserved_quantity.ge(qty)),
    )
    .set((
        available_quantity.eq(available_quantity + qty),
        reserved_quantity.eq(reserved_quantity - qty),
        updated_at.eq(chrono::Utc::now()),
    ))
    .get_result::<Stock>(conn);

    match affected {
        Ok(stock) => Ok(stock),
        Err(diesel::result::Error::NotFound) => {
            Err(AppError::Conflict(format!(
                "Cannot release {} units: reserved quantity insufficient for product {}",
                qty, prod_id
            )))
        }
        Err(err) => Err(map_diesel_error(err, "Stock")),
    }
}
