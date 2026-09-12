//! ==============================================================================
//! Orders Database Repository
//! ==============================================================================

use common::{AppResult};
use database::map_diesel_error;
use database::schema::orders::{order_items, orders};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::{NewOrder, NewOrderItem, Order, OrderItem};

pub fn insert_order(conn: &mut PgConnection, new_order: &NewOrder) -> AppResult<Order> {
    diesel::insert_into(orders::table)
        .values(new_order)
        .get_result::<Order>(conn)
        .map_err(|err| map_diesel_error(err, "Order"))
}

pub fn insert_order_items(
    conn: &mut PgConnection,
    items: &[NewOrderItem],
) -> AppResult<Vec<OrderItem>> {
    diesel::insert_into(order_items::table)
        .values(items)
        .get_results::<OrderItem>(conn)
        .map_err(|err| map_diesel_error(err, "OrderItem"))
}

pub fn find_order_by_id(conn: &mut PgConnection, order_id: Uuid) -> AppResult<Order> {
    orders::table
        .filter(orders::id.eq(order_id))
        .first::<Order>(conn)
        .map_err(|err| map_diesel_error(err, "Order"))
}

pub fn find_items_by_order_id(
    conn: &mut PgConnection,
    order_id: Uuid,
) -> AppResult<Vec<OrderItem>> {
    order_items::table
        .filter(order_items::order_id.eq(order_id))
        .load::<OrderItem>(conn)
        .map_err(|err| map_diesel_error(err, "OrderItem"))
}

pub fn update_order_status(
    conn: &mut PgConnection,
    order_id: Uuid,
    new_status: &str,
) -> AppResult<Order> {
    diesel::update(orders::table.filter(orders::id.eq(order_id)))
        .set((
            orders::status.eq(new_status),
            orders::updated_at.eq(chrono::Utc::now()),
        ))
        .get_result::<Order>(conn)
        .map_err(|err| map_diesel_error(err, "Order"))
}
