<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# Diesel SQL Types Reference Guide

In Diesel, type safety is achieved via a two-layer typing system:

1. **SQL Types (`diesel::sql_types::*`):** Phantom marker types representing database column types in the query builder and in `schema.rs`.
2. **Rust Types:** Concrete standard library or third-party crate types (such as `i32`, `String`, `chrono::DateTime`, `uuid::Uuid`) that implement the `FromSql` and `ToSql` traits for the corresponding SQL type.

Below is the comprehensive catalog of SQL types provided by Diesel, categorized by domain and backend availability.

## 1. Core / Universal Types (Available Across SQLite, MySQL, and PostgreSQL)

These fundamental types exist across all supported backends.

| Diesel SQL Type (`diesel::sql_types::*`)  | Rust Primitive / Standard Type | Typical SQL Type              | Description                                              |
| ----------------------------------------- | ------------------------------ | ----------------------------- | -------------------------------------------------------- |
| `Bool`                                    | `bool`                         | `BOOLEAN` / `TINYINT(1)`      | Logical boolean value (`true` / `false`).                |
| `SmallInt`                                | `i16`                          | `SMALLINT` / `INT2`           | Signed 16-bit integer (-32,768 to 32,767).               |
| `Integer`                                 | `i32`                          | `INTEGER` / `INT` / `INT4`    | Standard signed 32-bit integer.                          |
| `BigInt`                                  | `i64`                          | `BIGINT` / `INT8`             | Signed 64-bit integer.                                   |
| `Float`                                   | `f32`                          | `REAL` / `FLOAT4`             | Single-precision 32-bit floating-point number.           |
| `Double`                                  | `f64`                          | `DOUBLE PRECISION` / `FLOAT8` | Double-precision 64-bit floating-point number.           |
| `Text`                                    | `String`, `&str`               | `TEXT` / `VARCHAR`            | Variable-length UTF-8 encoded text strings.              |
| `Binary`                                  | `Vec<u8>`, `&[u8]`             | `BYTEA` / `BLOB`              | Raw binary byte buffers.                                 |
| `Nullable<T>`                             | `Option<T>`                    | Any nullable column           | Marker wrapper indicating a column allows `NULL` values. |

## 2. Date and Time Types

These types require enabling third-party integrations in `Cargo.toml` (typically the `chrono` or `time` feature flags on `diesel`).

| Diesel SQL Type (`diesel::sql_types::*`) | Rust Type (`chrono`)                 | Typical SQL Type         | Description                                            |
| ---------------------------------------- | ------------------------------------ | ------------------------ | ------------------------------------------------------ |
| `Date`                                   | `chrono::NaiveDate`                  | `DATE`                   | Calendar date without a time-of-day component.         |
| `Time`                                   | `chrono::NaiveTime`                  | `TIME`                   | Time of day without timezone information.              |
| `Timestamp`                              | `chrono::NaiveDateTime`              | `TIMESTAMP` / `DATETIME` | Date and time without timezone offset.                 |
| `Timestamptz` *(PostgreSQL only)*        | `chrono::DateTime<Utc>`              | `TIMESTAMPTZ`            | Timestamp with timezone offset, stored in UTC.         |
| `Interval` *(PostgreSQL only)*           | `diesel::pg::data_types::PgInterval` | `INTERVAL`               | A span of elapsed time (years, months, days, seconds). |

## 3. High-Precision Numeric Types

Used for financial calculations, monetary amounts, and arbitrary-precision data.

| Diesel SQL Type (`diesel::sql_types::*`) | Rust Type                                           | Typical SQL Type     | Description                                                                                                                                                   |
| ---------------------------------------- | --------------------------------------------------- | -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Numeric`                                | `bigdecimal::BigDecimal` or `rust_decimal::Decimal` | `NUMERIC` / `DECIMAL`| Arbitrary-precision exact fixed-point decimal. Avoids floating-point rounding errors. Requires `numeric-bigdecimal` or `numeric-rust-decimal` crate features. |
| `Money` *(PostgreSQL only)*              | `diesel::pg::data_types::PgMoney`                   | `MONEY`              | Currency and cash amounts with fixed fractional precision.                                                                                                    |

## 4. PostgreSQL-Specific Types

PostgreSQL includes specialized data types that Diesel models out-of-the-box when compiled with the `postgres` feature flag.

### A. UUIDs, Networks, and Geometric Data

| Diesel SQL Type (`diesel::pg::sql_types::*`) | Rust Type                  | SQL Type  | Description                                                        |
| -------------------------------------------- | -------------------------- | --------- | ------------------------------------------------------------------ |
| `Uuid`                                       | `uuid::Uuid`               | `UUID`    | 128-bit universally unique identifiers (requires `uuid` feature).  |
| `Inet`                                       | `ipnetwork::IpNetwork`     | `INET`    | IPv4 or IPv6 network host address or subnet.                       |
| `Cidr`                                       | `ipnetwork::IpNetwork`     | `CIDR`    | IPv4 or IPv6 network specification.                                |
| `MacAddr`                                    | `[u8; 6]`                  | `MACADDR` | 48-bit media access control (hardware) address.                    |
| `Point`                                      | `(f64, f64)`               | `POINT`   | Geometric point on a two-dimensional plane ($x, y$).               |
| `Box`                                        | `((f64, f64), (f64, f64))` | `BOX`     | Rectangular box defined by pairs of opposite corners.              |

### B. Structured Documents (JSON)

| Diesel SQL Type (`diesel::pg::sql_types::*`) | Rust Type           | SQL Type | Description                                                                          |
| -------------------------------------------- | ------------------- | -------- | ------------------------------------------------------------------------------------ |
| `Json`                                       | `serde_json::Value` | `JSON`   | Textual JSON representation. Retains whitespace and key order.                       |
| `Jsonb`                                      | `serde_json::Value` | `JSONB`  | Decomposed binary-format JSON. Supports indexed lookup operations (`@>`, `?`, etc.). |

### C. Full-Text Search

| Diesel SQL Type (`diesel::pg::sql_types::*`) | Rust Type                  | SQL Type   | Description                                                     |
| -------------------------------------------- | -------------------------- | ---------- | --------------------------------------------------------------- |
| `TsVector`                                   | `String` / Internal handle | `TSVECTOR` | Sorted list of distinct lexemes optimized for full-text search. |
| `TsQuery`                                    | `String` / Internal handle | `TSQUERY`  | Lexemes to be searched with boolean operators (`&` `|` `!`).    |

### D. Arrays and Ranges

| Diesel SQL Type (`diesel::pg::sql_types::*`) | Rust Type              | SQL Type                                     | Description                                                        |
| -------------------------------------------- | ---------------------- | -------------------------------------------- | ------------------------------------------------------------------ |
| `Array<T>`                                   | `Vec<T>`               | `T[]`                                        | Homogeneous multidimensional array of another Diesel SQL type `T`. |
| `Range<T>`                                   | `(Bound<T>, Bound<T>)` | Range Types (`int4range`, `daterange`, etc.) | Continuous range of values with upper and lower bounds.            |

## 5. MySQL-Specific Types

Types tailored to the MySQL storage engine and conventions.

| Diesel SQL Type (`diesel::mysql::sql_types::*`) | Rust Type               | SQL Type          | Description                                           |
| ----------------------------------------------- | ----------------------- | ----------------- | ----------------------------------------------------- |
| `Unsigned<SmallInt>`                            | `u16`                   | SMALLINT UNSIGNED | Unsigned 16-bit integer (0 to 65,535).                |
| `Unsigned<Integer>`                             | `u32`                   | INTEGER UNSIGNED  | Unsigned 32-bit integer.                              |
| `Unsigned<BigInt>`                              | `u64`                   | BIGINT UNSIGNED   | Unsigned 64-bit integer.                              |
| `Datetime`                                      | `chrono::NaiveDateTime` | DATETIME          | MySQL combined date and time format without timezone. |
| `Year`                                          | `i32`                   | YEAR              | 4-digit year representation (1901 to 2155).           |

## 6. Modifier and Macro Types

These types are not standalone SQL column definitions; they act as operators or markers within query expressions:

* **`Nullable<ST>`**: Wraps any SQL type `ST` to allow `NULL` states. When selected, it maps directly to `Option<T>` in Rust structs.
* **`Array<ST>`**: Wraps any SQL type `ST` into a PostgreSQL array column, mapping to `Vec<T>` in Rust.
* **`Untyped`**: Used internally by Diesel when raw SQL or fragments bypass type checking.
* **Custom Enums via `#[derive(diesel_derive_enum::DbEnum)]` or `diesel::sql_types::SqlType**`: Custom PostgreSQL `ENUM` types map to user-defined Rust enums via derive macros.

## Practical Example: Schema Mapping

Here is how these types translate from database DDL to Diesel's `schema.rs` and down to your application structs:

```sql
-- PostgreSQL DDL
CREATE TABLE catalog.products (
    id UUID PRIMARY KEY,
    sku VARCHAR NOT NULL,
    price_cents BIGINT NOT NULL,
    metadata JSONB,
    tags TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);
```

```rust
// crates/database/src/schema.rs (Generated by Diesel CLI)
diesel::table! {
    catalog.products (id) {
        id -> Uuid,
        sku -> Varchar,
        price_cents -> Int8,
        metadata -> Nullable<Jsonb>,
        tags -> Array<Text>,
        created_at -> Timestamptz,
    }
}
```

```rust
// Rust Model (Your application code)
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::catalog::products)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub price_cents: i64,
    pub metadata: Option<Value>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}
```

---
