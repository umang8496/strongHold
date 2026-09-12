//! ==============================================================================
//! Catalog Product Repository
//! ==============================================================================
//! Encapsulates all raw Diesel SQL operations for the `catalog.products` table.
//! Methods here receive an active mutable reference to a `PgConnection` and return
//! domain models or domain AppResult errors.
//! ==============================================================================

use common::{AppError, AppResult};
use database::map_diesel_error;
use database::schema::catalog::products::dsl::*;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::{NewProduct, Product, UpdateProduct};

/// Inserts a brand new product into the database and returns the generated entity.
///
/// # Arguments
/// * `conn` - An active mutable connection to the PostgreSQL database.
/// * `new_product` - The insertable payload struct containing title, sku, price, etc.
pub fn insert_product(conn: &mut PgConnection, new_product: &NewProduct) -> AppResult<Product> {
    diesel::insert_into(products)
        .values(new_product)
        // `get_result` appends `RETURNING *` to the SQL INSERT statement,
        // allowing Postgres to return the generated UUID and DEFAULT timestamps
        // in a single round-trip without a second SELECT query.
        .get_result::<Product>(conn)
        .map_err(|err| map_diesel_error(err, "Product"))
}

/// Finds an active product by its unique UUID primary key.
pub fn find_product_by_id(conn: &mut PgConnection, product_id: Uuid) -> AppResult<Product> {
    products
        .filter(id.eq(product_id))
        // Enforce that soft-deleted or inactive items are excluded
        .filter(is_active.eq(true))
        .first::<Product>(conn)
        .map_err(|err| map_diesel_error(err, "Product"))
}

/// Finds a product by its unique SKU (Stock Keeping Unit).
pub fn find_product_by_sku(conn: &mut PgConnection, product_sku: &str) -> AppResult<Product> {
    products
        .filter(sku.eq(product_sku))
        .first::<Product>(conn)
        .map_err(|err| map_diesel_error(err, "Product"))
}

/// Lists products with basic pagination and price filtering.
///
/// Notice the use of Diesel's `.into_boxed()`:
/// In Rust, static query types are resolved at compile time. `.into_boxed()` allows us
/// to conditionally attach dynamic SQL WHERE clauses (e.g., if min_price is present)
/// without violating Rust's strict static typing.
pub fn list_products(
    conn: &mut PgConnection,
    page: i64,
    page_size: i64,
    min_price: Option<i64>,
    max_price: Option<i64>,
) -> AppResult<Vec<Product>> {
    let mut query = products
        .filter(is_active.eq(true))
        .into_boxed();

    if let Some(min) = min_price {
        query = query.filter(price_cents.ge(min));
    }

    if let Some(max) = max_price {
        query = query.filter(price_cents.le(max));
    }

    let offset = (page.max(1) - 1) * page_size.max(1);

    query
        .order(created_at.desc())
        .limit(page_size.clamp(1, 100))
        .offset(offset)
        .load::<Product>(conn)
        .map_err(|err| map_diesel_error(err, "Product"))
}

/// Updates an existing product using Diesel's `AsChangeset`.
pub fn update_product(
    conn: &mut PgConnection,
    product_id: Uuid,
    changes: &UpdateProduct,
) -> AppResult<Product> {
    diesel::update(products.filter(id.eq(product_id)))
        .set(changes)
        .get_result::<Product>(conn)
        .map_err(|err| map_diesel_error(err, "Product"))
}

/// Soft-deletes a product by flipping `is_active` to false.
///
/// In e-commerce systems, products should rarely be physically deleted (`DELETE FROM`)
/// because historic orders reference them. Soft-deleting preserves foreign key integrity.
pub fn soft_delete_product(conn: &mut PgConnection, product_id: Uuid) -> AppResult<()> {
    let rows_affected = diesel::update(products.filter(id.eq(product_id)))
        .set(is_active.eq(false))
        .execute(conn)
        .map_err(|err| map_diesel_error(err, "Product"))?;

    if rows_affected == 0 {
        return Err(AppError::NotFound("Product not found".to_string()));
    }

    Ok(())
}
