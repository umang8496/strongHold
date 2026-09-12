<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# Diesel Schema Reference (`schema.rs`)

This document tracks and explains every construct generated in `crates/database/src/schema.rs`.  
Update this file whenever new migrations alter the schema.

## File Header & Namespaces

* **`// @generated automatically by Diesel CLI.`**  
Tells linters (`clippy`), formatters (`rustfmt`), and IDEs to ignore this file and warns developers that manual edits will be overwritten during `diesel migration run`.
* **`pub mod catalog { ... }`**  
Mirrors the PostgreSQL namespace `catalog`.  
Prevents table collisions across domains (e.g., `catalog::products` vs. `orders::products`).

### Macro & Table Directives

* **`diesel::table! { ... }`**  
Procedural macro that generates Rust types, unit structs, and relational traits (`Table`, `Column`, `Expression`, `Selectable`) representing the database table to `rustc`.
* **`use diesel::sql_types::*;`**  
Brings all Diesel SQL marker types (`Uuid`, `Varchar`, `Int8`, etc.) into the macro scope so column types resolve without full paths.
* **`catalog.products (id)`**  
* `catalog.products`: Defines the fully qualified table name for queries (`FROM catalog.products`).
* `(id)`: Declares the Primary Key. Enables compile-time generation of `.find(id)` lookups.

### Column Definitions (`catalog.products`)

* **`id -> Uuid`**  
Primary key. Maps to `uuid::Uuid`.  
Provides collision-free IDs that can be generated client-side or domain-side before database insertion.
* **`#[max_length = 64]` / `#[max_length = 255]`**  
Length metadata extracted from PostgreSQL `VARCHAR(n)`.  
Used by Diesel for compile-time or runtime string validation.
* **`sku -> Varchar`**  
Unique product inventory code. Maps to `String`.
* **`title -> Varchar`**  
Display name. Maps to `String`.
* **`description -> Nullable<Text>`**  
Optional detailed copy.  
Wrapped in `Nullable<...>` because the column lacks a `NOT NULL` constraint.  
**Must** map to `Option<String>` in Rust structs; mapping to plain `String` causes a compile error.
* **`price_cents -> Int8`**  
64-bit integer (`i64`).  
Represents the price in the lowest currency unit (cents) to eliminate floating-point rounding errors (`f32`/`f64`).
* **`is_active -> Bool`**  
Maps to `bool`. Enables soft-deletion and visibility toggles without dropping rows.
* **`created_at -> Timestamptz` / `updated_at -> Timestamptz`**  
Maps to `chrono::DateTime<chrono::Utc>`.  
Enforces timezone-aware storage (stored as UTC in PostgreSQL) to avoid timezone conversion bugs.

---
