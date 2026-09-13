<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# Deep-Dive Engineering Guide: Diesel ORM, Connections, Transactions, and Complex Query Construction in Rust

## 1. How Diesel Connects to the Database

Diesel is a compile-time checked ORM and query builder for Rust.  
Unlike pure Rust network clients (such as `tokio-postgres`), Diesel's PostgreSQL backend (`diesel::pg::PgConnection`) relies on the native PostgreSQL C client library: **`libpq`**.

### The Connection Pipeline

```text
[Rust Application Code]
         │
         ▼
[Diesel DSL / PgConnection API]
         │  (C FFI Bindings via `pq-sys`)
         ▼
[libpq Native C Shared Library]  <── Linked via OS dynamic linker
         │
         ▼  (PostgreSQL Frontend/Backend Protocol v3.0 over TCP/Unix Socket)
[PostgreSQL Database Server]
```

#### The Role of `libpq` and Native Linking

When `diesel` compiles with the `postgres` feature flag:

1. Diesel depends on the low-level `pq-sys` crate.
2. `pq-sys` dynamically binds to `libpq.so` (Linux) or `libpq.dylib` (macOS).
3. `libpq` manages network sockets, TLS handshakes, message framing, parameter encoding, and low-level protocol state machines.

#### Unpooled Connection vs. Connection Pooling

Diesel provides a direct connection primitive:

```rust
use diesel::pg::PgConnection;
use diesel::prelude::*;

// Direct, unpooled TCP connection establishment
let mut conn = PgConnection::establish("postgres://user:pass@localhost:5432/my_db")
    .expect("Failed to connect to database");
```

Direct connections introduce severe latency penalties in web services: every HTTP request would incur TCP handshake, TLS negotiation, authentication, and backend process forking on PostgreSQL.

Production services pair Diesel with a connection pool like **`r2d2`**:

- Diesel implements `r2d2::ManageConnection` via `diesel::r2d2::ConnectionManager<PgConnection>`.
- The connection pool manages idle connection timeouts, connection health validation (`SELECT 1`), and concurrent thread checkout queues.

```rust
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_pool(database_url: &str, max_size: u32) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("Failed to create connection pool")
}
```

#### Blocking I/O and Asynchronous Runtimes

`libpq` and Diesel 2.x are strictly **synchronous (blocking)**.

Executing a blocking query inside an `async fn` on an Actix-web or Tokio runtime stalls that runtime worker thread, preventing it from processing other concurrent tasks.  
Diesel queries must be wrapped in `actix_web::web::block` or `tokio::task::spawn_blocking`:

```rust
use actix_web::web;

let result = web::block(move || {
    let mut conn = pool.get()?;
    // Blocking Diesel query runs safely on Tokio's blocking thread pool
    products::table.load::<Product>(&mut conn)
}).await?;
```

## 2. Diesel Architecture & Compile-Time Safety Model

Diesel’s central design goal is eliminating SQL runtime bugs at compile time.

### How `schema.rs` Works (`table!` Macro)

When running `diesel print-schema`, Diesel inspects the database catalog tables (`information_schema.columns`) and generates type definitions:

```rust
diesel::table! {
    catalog.products (id) {
        id -> Uuid,
        #[max_length = 64]
        sku -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        price_cents -> Int8,
        is_active -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}
```

This macro generates:

1. A zero-sized struct named `products::table`.
2. A unit struct for each column (`products::id`, `products::sku`, etc.).
3. Trait implementations (`Column`, `Expression`, `SelectableExpression`, `Table`) establishing what can be selected, filtered, or joined.

### Trait Derivations for Models

Diesel splits data models into specific operational lifecycles:

| Trait                     | Purpose                                                   | Usage Context                                                                                                                                                          |
| ------------------------- | --------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `#[derive(Queryable)]`    | Deserializes raw SQL result sets into a struct.           | Used with `.load()`, `.first()`, `.get_result()`. Requires exact field-order and field-type matching with the query select list.                                       |
| `#[derive(Selectable)]`   | Enables typed column projection via `Model::as_select()`. | Allows struct-driven projections instead of relying on column index order.                                                                                             |
| `#[derive(Insertable)]`   | Maps struct fields into an SQL `INSERT INTO` clause.      | Used with `diesel::insert_into(table).values(&model)`. Omits generated columns like auto-generated UUIDs or defaults.                                                  |
| `#[derive(AsChangeset)]`  | Maps fields to `UPDATE table SET col = val`.              | Used with `diesel::update(...).set(&changeset)`. `Option<T>` fields allow updating only populated values when configured with `#[diesel(treat_none_as_null = false)]`. |
| `#[derive(Identifiable)]` | Associates a model with its primary key (`id`).           | Simplifies lookups and updates: `diesel::update(&product).set(...)`.                                                                                                   |

```rust
#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = database::schema::catalog::products)]
pub struct Product {
    pub id: uuid::Uuid,
    pub sku: String,
    pub title: String,
    pub description: Option<String>, // Nullable in SQL maps to Option in Rust
    pub price_cents: i64,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

## 3. Basic CRUD Operations

### Create (Insert)

Diesel supports inserting single records, batch inserting slices, and returning the persisted database state via PostgreSQL's native `RETURNING` clause:

```rust
use database::schema::catalog::products::dsl::*;
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = database::schema::catalog::products)]
pub struct NewProduct<'a> {
    pub sku: &'a str,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub price_cents: i64,
    pub is_active: bool,
}

// Single insert returning full inserted row (single network round-trip)
pub fn create_product(conn: &mut PgConnection, item: &NewProduct) -> QueryResult<Product> {
    diesel::insert_into(products)
        .values(item)
        .get_result(conn) // Appends 'RETURNING *' under the hood
}

// Batch insert returning count of inserted rows
pub fn create_products_batch(conn: &mut PgConnection, items: &[NewProduct]) -> QueryResult<usize> {
    diesel::insert_into(products)
        .values(items)
        .execute(conn)
}
```

### Read (Query)

Querying leverages strongly typed builders:

```rust
// Load first matching row (LIMIT 1)
pub fn get_by_id(conn: &mut PgConnection, target_id: uuid::Uuid) -> QueryResult<Product> {
    products
        .filter(id.eq(target_id))
        .filter(is_active.eq(true))
        .first::<Product>(conn)
}

// Select only specific columns using Selectable projection
pub fn get_title_and_price(conn: &mut PgConnection, target_id: uuid::Uuid) -> QueryResult<(String, i64)> {
    products
        .filter(id.eq(target_id))
        .select((title, price_cents))
        .first(conn)
}

// Load all matching rows
pub fn get_active_products(conn: &mut PgConnection) -> QueryResult<Vec<Product>> {
    products
        .filter(is_active.eq(true))
        .order(created_at.desc())
        .load::<Product>(conn)
}
```

### Update

Updates use `diesel::update`:

```rust
#[derive(AsChangeset)]
#[diesel(table_name = database::schema::catalog::products)]
#[diesel(treat_none_as_null = false)]
pub struct UpdateProductChanges<'a> {
    pub title: Option<&'a str>,
    pub price_cents: Option<i64>,
}

// Update via Changeset
pub fn update_product(
    conn: &mut PgConnection,
    target_id: uuid::Uuid,
    changes: &UpdateProductChanges,
) -> QueryResult<Product> {
    diesel::update(products.filter(id.eq(target_id)))
        .set(changes)
        .get_result(conn)
}

// Targeted single-column atomic expression update
pub fn increment_price(conn: &mut PgConnection, target_id: uuid::Uuid, delta: i64) -> QueryResult<usize> {
    diesel::update(products.filter(id.eq(target_id)))
        .set(price_cents.eq(price_cents + delta))
        .execute(conn)
}
```

### Delete (Hard vs. Soft)

```rust
// Hard Delete: Physically removes the row
pub fn hard_delete_product(conn: &mut PgConnection, target_id: uuid::Uuid) -> QueryResult<usize> {
    diesel::delete(products.filter(id.eq(target_id))).execute(conn)
}

// Soft Delete: Sets lifecycle flag, preserving historic order references
pub fn soft_delete_product(conn: &mut PgConnection, target_id: uuid::Uuid) -> QueryResult<usize> {
    diesel::update(products.filter(id.eq(target_id)))
        .set(is_active.eq(false))
        .execute(conn)
}
```

## 4. Database Transactions in Diesel

Transactions are executed using closures on `PgConnection`.

### Basic Transaction Mechanics

When calling `conn.transaction(closure)`:

1. Diesel runs `BEGIN`.
2. The closure receives `&mut PgConnection` or `&mut TxConn`.
3. If the closure returns `Ok(value)`, Diesel executes `COMMIT`.
4. If the closure returns `Err(error)`, Diesel executes `ROLLBACK`.

```rust
use diesel::Connection;

pub fn transfer_balance(
    conn: &mut PgConnection,
    from_user: uuid::Uuid,
    to_user: uuid::Uuid,
    amount: i64,
) -> Result<(), AppError> {
    conn.transaction::<(), AppError, _>(|tx_conn| {
        // Step 1: Deduct from source
        let rows = diesel::update(wallets::table.filter(wallets::user_id.eq(from_user)))
            .set(wallets::balance.eq(wallets::balance - amount))
            .execute(tx_conn)?;

        if rows == 0 {
            return Err(AppError::NotFound("Source user wallet missing".into()));
        }

        // Step 2: Add to destination
        diesel::update(wallets::table.filter(wallets::user_id.eq(to_user)))
            .set(wallets::balance.eq(wallets::balance + amount))
            .execute(tx_conn)?;

        Ok(())
    })
}
```

### Error Trait Bound Requirement (`From<diesel::result::Error>`)

The transaction closure enforces that the returned error type implements `From<diesel::result::Error>`:

$$\text{Error Type } E: \text{From}\langle\text{diesel::result::Error}\rangle$$

If PostgreSQL returns a network drop or constraint violation during `COMMIT` or `ROLLBACK`, Diesel must return that failure wrapped inside your application error type.

### Savepoints and Nested Transactions

Diesel supports nested transactions using SQL **savepoints**:

```rust
conn.transaction::<(), AppError, _>(|outer_tx| {
    // Top-level BEGIN issued

    outer_tx.transaction::<(), AppError, _>(|nested_tx| {
        // Diesel detects an existing transaction and executes:
        // SAVEPOINT diesel_savepoint_1;
        // If this closure fails, it runs:
        // ROLLBACK TO SAVEPOINT diesel_savepoint_1;
        Ok(())
    })?;

    Ok(())
})
```

## 5. Complex Queries in Rust with Diesel

### 5.1 Dynamic Filtering with `.into_boxed()`

Rust’s compiler requires static typing for every query builder chain.  
Adding conditional `.filter()` calls changes the Rust return type, which causes type mismatches.

`.into_boxed()` moves the internal AST to the heap, converting concrete static query types into a boxed query (`BoxedSelectStatement`) that accepts dynamic filters:

```rust
pub struct ProductFilter {
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub sku_search: Option<String>,
}

pub fn search_products(
    conn: &mut PgConnection,
    filter: ProductFilter,
) -> QueryResult<Vec<Product>> {
    let mut query = products::table
        .filter(products::is_active.eq(true))
        .into_boxed();

    if let Some(min) = filter.min_price {
        query = query.filter(products::price_cents.ge(min));
    }

    if let Some(max) = filter.max_price {
        query = query.filter(products::price_cents.le(max));
    }

    if let Some(ref search) = filter.sku_search {
        query = query.filter(products::sku.ilike(format!("%{}%", search)));
    }

    query.load::<Product>(conn)
}
```

### 5.2 Joins (Inner, Left, Cross-Schema)

To join tables, their relationship must be declared using `diesel::joinable!`.

#### Join Registration

In `crates/database/src/schema.rs`:

```rust
diesel::joinable!(orders::order_items -> orders::orders (order_id));
```

#### Executing Typed Joins

```rust
use database::schema::orders::{orders, order_items};

// Inner Join (Returns tuple of matching structs)
pub fn get_order_with_items(
    conn: &mut PgConnection,
    target_order_id: uuid::Uuid,
) -> QueryResult<(Order, Vec<OrderItem>)> {
    let raw_rows = orders::table
        .inner_join(order_items::table)
        .filter(orders::id.eq(target_order_id))
        .load::<(Order, OrderItem)>(conn)?;

    let order = raw_rows.first().ok_or(diesel::result::Error::NotFound)?.0.clone();
    let items = raw_rows.into_iter().map(|(_, item)| item).collect();

    Ok((order, items))
}

// Left Join (Second model wrapped in Option<T>)
pub fn get_order_optional_items(
    conn: &mut PgConnection,
) -> QueryResult<Vec<(Order, Option<OrderItem>)>> {
    orders::table
        .left_join(order_items::table)
        .load::<(Order, Option<OrderItem>)>(conn)
}
```

### 5.3 Aggregations and `GROUP BY`

SQL aggregations (`COUNT`, `SUM`, `AVG`, `MIN`, `MAX`) require grouping non-aggregated select columns:

```rust
use diesel::dsl::{count, sum};

pub struct OrderSummary {
    pub customer_id: uuid::Uuid,
    pub total_spent: i64,
    pub total_orders: i64,
}

pub fn get_customer_order_metrics(
    conn: &mut PgConnection,
) -> QueryResult<Vec<OrderSummary>> {
    let rows = orders::table
        .group_by(orders::customer_id)
        .select((
            orders::customer_id,
            sum(orders::total_amount_cents),
            count(orders::id),
        ))
        .load::<(uuid::Uuid, Option<i64>, i64)>(conn)?;

    let summaries = rows
        .into_iter()
        .map(|(cust_id, total_spent, total_count)| OrderSummary {
            customer_id: cust_id,
            total_spent: total_spent.unwrap_or(0),
            total_orders: total_count,
        })
        .collect();

    Ok(summaries)
}
```

### 5.4 Subqueries

Diesel allows using subqueries inside expressions (such as `eq_any` or scalar comparisons):

```rust
// Find all products that have never appeared in any order item
pub fn get_unpurchased_products(conn: &mut PgConnection) -> QueryResult<Vec<Product>> {
    let purchased_product_ids = order_items::table
        .select(order_items::product_id)
        .distinct();

    products::table
        .filter(products::id.ne_all(purchased_product_ids))
        .load::<Product>(conn)
}
```

### 5.5 Pessimistic Locking (`FOR UPDATE` / `SKIP LOCKED`)

To prevent concurrent checkout threads from double-allocating inventory, use PostgreSQL row-level locks:

```rust
pub fn lock_stock_for_update(
    conn: &mut PgConnection,
    target_product_id: uuid::Uuid,
) -> QueryResult<Stock> {
    use database::schema::inventory::stocks::dsl::*;

    stocks
        .filter(product_id.eq(target_product_id))
        // Issues: SELECT * FROM inventory.stocks WHERE ... FOR UPDATE
        .for_update()
        .first::<Stock>(conn)
}

// High-throughput concurrency queue consumer: SKIP LOCKED
pub fn claim_job_batch(conn: &mut PgConnection, limit: i64) -> QueryResult<Vec<Stock>> {
    use database::schema::inventory::stocks::dsl::*;

    stocks
        .for_update()
        .skip_locked()
        .limit(limit)
        .load::<Stock>(conn)
}
```

### 5.6 Raw SQL Fallback (`sql_query`)

When an analytical query uses PostgreSQL features not supported by Diesel's DSL (such as recursive CTEs, window functions like `ROW_NUMBER() OVER (...)`, or specific JSONB operators),  
drop down to type-safe raw SQL via `diesel::sql_query`.  

To deserialize raw SQL results, annotate a struct with `#[derive(QueryableByName)]`:

```rust
use diesel::sql_types::{BigInt, Uuid as DieselUuid};
use diesel::sql_query;

#[derive(QueryableByName, Debug)]
pub struct CategorySalesRank {
    #[diesel(sql_type = DieselUuid)]
    pub product_id: uuid::Uuid,
    #[diesel(sql_type = BigInt)]
    pub total_sales_cents: i64,
    #[diesel(sql_type = BigInt)]
    pub category_rank: i64,
}

pub fn get_category_sales_rankings(conn: &mut PgConnection) -> QueryResult<Vec<CategorySalesRank>> {
    sql_query(
        "WITH ranked_sales AS (
            SELECT 
                product_id,
                SUM(unit_price_cents * quantity) AS total_sales_cents,
                DENSE_RANK() OVER (ORDER BY SUM(unit_price_cents * quantity) DESC) AS category_rank
            FROM orders.order_items
            GROUP BY product_id
        )
        SELECT product_id, total_sales_cents, category_rank 
        FROM ranked_sales 
        WHERE category_rank <= 10"
    )
    .load::<CategorySalesRank>(conn)
}
```

#### Parameterized Raw Queries

Prevent SQL injection using the `bind` method:

```rust
pub fn get_user_orders_raw(
    conn: &mut PgConnection,
    cust_id: uuid::Uuid,
    min_amount: i64,
) -> QueryResult<Vec<Order>> {
    sql_query("SELECT * FROM orders.orders WHERE customer_id = $1 AND total_amount_cents >= $2")
        .bind::<DieselUuid, _>(cust_id)
        .bind::<BigInt, _>(min_amount)
        .load::<Order>(conn)
}
```

## 6. Performance and Production Patterns

### Prepared Statements

Diesel prepares statements automatically per connection.  
If a statement runs multiple times on the same connection, `libpq` bypasses query plan parsing and goes straight to execution.

### Inspection via `diesel::debug_query`

To inspect the SQL generated by Diesel without running it against the database:

```rust
let query = products::table
    .filter(products::price_cents.gt(1000))
    .order(products::created_at.desc());

// Print generated SQL string
let sql = diesel::debug_query::<diesel::pg::Pg, _>(&query).to_string();
println!("Generated SQL: {}", sql);

```

### Best Practices Checklist

- **Connection Lifecycle:** Never hold a connection across an `.await` boundary in async code. Check out the connection, run the query inside `web::block`, and let it return to the pool immediately.
- **Projections:** Use `.select(Model::as_select())` or tuple selections rather than full table loads to avoid reading unused columns (such as large text blobs).
- **Schema Segregation:** Run `diesel print-schema --schema <domain>` for each Postgres schema to keep generated modules cleanly partitioned across workspace crates.

---
