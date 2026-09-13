<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# Rust Library APIs

- [Macro [actix_web::main]](#macro-actix_webmain)
- [Macro [derive(...)]](#macro-attribute-derive)
- [Rust `Result<T, E>` and `Option<T>` enums](#rust-resultt-e-and-optiont-enums)
<!-- - []() -->
<!-- - []() -->

---

## Macro `[actix_web::main]`

OS does not know how to run an asynchronous function. The native OS entry point for any executable must always be a synchronous function:

```rust
fn main() -> std::io::Result<()> // Synchronous OS entry point
```

Rust does not bundle an asynchronous runtime into its standard library.  
If we write:

```rust
// COMPILE ERROR: `main` function is not allowed to be `async`
async fn main() -> std::io::Result<()> {
    HttpServer::new(...).bind(...)?
        .run()
        .await
}
```

The compiler rejects this because `.await` requires an active executor (an async runtime) to poll futures, handle task scheduling, and drive non-blocking I/O events.

`#[actix_web::main]` rewrites the `async fn main()` into a standard synchronous `fn main()` at compile time.  
It initializes an `actix_rt::System` (Actix's Tokio-backed runtime) and blocks the main thread until your async code finishes.  

It is an attribute macro that turns the `async fn main()` into a standard, synchronous entry point required by the OS.

### The Problem

Rust's compiler and the operating system do not support native `async fn main()`.  
Rust has no built-in async runtime, so `.await` cannot execute without an external executor.  

### Code Expansion

**What you write:**

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(...).bind("127.0.0.1:8080")?.run().await
}
```

**What the compiler sees:**

```rust
fn main() -> std::io::Result<()> {
    actix_rt::System::new().block_on(async move {
        HttpServer::new(...).bind("127.0.0.1:8080")?.run().await
    })
}
```

### Core Responsibilities

- **Boots the Runtime:** Initializes `actix-rt` (an event-loop runtime built on top of Tokio).
- **Bridges Sync to Async:** Calls `.block_on(...)` to block the main OS thread until the HTTP server shuts down.
- **Sets up Threading:** Provisions the single-threaded-per-core event loops Actix uses for high-throughput HTTP handling.

---

## Macro attribute `[derive(...)]`

It is a procedural macro attribute that tells the compiler to automatically generate boilerplate implementations of specific traits for your `struct` or `enum`.  

Instead of writing manual `impl Trait for Type` blocks, the compiler writes them for you during compilation.

### Key Traits in `order-engine`

#### `Debug`

- **Purpose:** Enables string formatting using `{:?}` (logging, panics, `assert_eq!`).
- **Under the Hood:** Implements `fmt::Debug` to print the type name and field values.

```rust
#[derive(Debug)]
pub struct Product {
    pub sku: String,
    pub price_cents: i64,
}
```

or

```rust
pub struct Product {
    pub sku: String,
    pub price_cents: i64,
}

impl std::fmt::Debug for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Product")
            .field("sku", &self.sku)
            .field("price_cents", &self.price_cents)
            .finish()
    }
}
```

#### `Clone`

- **Purpose:** Explicit, potentially expensive duplication via `.clone()`.
- **Under the Hood:** Recursively calls `.clone()` on every field (e.g., allocating a new `String` or cloning a reference-counted `Arc`).

#### `Copy`

- **Purpose:** Implicit, bitwise duplication (C-style `memcpy`).
- **Rule:** Requires `Clone`. Only valid for types that live entirely on the stack and own no heap pointers (e.g., `OrderStatus` enum, `i32`, `bool`).
- **Under the Hood:** Assignment (`a = b`) duplicates bits rather than transferring ownership.

```rust
#[derive(Clone, Copy)]
pub enum OrderStatus {
    Draft,
    Completed,
}
```

or

```rust
pub enum OrderStatus {
    Draft,
    Completed,
}

// 1. Clone implementation: explicit copy
impl Clone for OrderStatus {
    fn clone(&self) -> Self {
        *self // For enums/primitive fields, returns a bitwise copy
    }
}

// 2. Copy implementation: marker trait enabling implicit assignment/memcpy
impl Copy for OrderStatus {}
```

#### `serde::Serialize`

- **Purpose:** Converts Rust structs into external formats (JSON, URL queries).
- **Under the Hood:** Generates a state machine that traverses struct fields and emits tokens for serializer backends (e.g., generating JSON via `serde_json`).

#### `serde::Deserialize`

- **Purpose:** Parses external input (HTTP JSON request bodies) into typed Rust structs.
- **Under the Hood:** Generates an internal `Visitor` that validates types, checks required fields, and maps missing nullable fields to `None`.

#### `thiserror::Error`

- **Purpose:** Transforms an `enum` into a production-ready error type.
- **Under the Hood:** Automatically implements `std::error::Error` and `Display` using the messages in `#[error("...")]`, and implements `From<E>` for variants marked with `#[from]`.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),
}
```

or

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Database(diesel::result::Error),
}

// 1. Manually implement Display (what #[error("...")] generates)
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Resource not found: {}", msg),
            AppError::Database(err) => write!(f, "Database error: {}", err),
        }
    }
}

// 2. Manually implement std::error::Error trait
impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::NotFound(_) => None,
            AppError::Database(err) => Some(err), // chain the underlying cause
        }
    }
}

// 3. Manually implement From for the ? operator (what #[from] generates)
impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        AppError::Database(err)
    }
}
```

### Quick Reference

| Trait         | Origin            | Mechanism                      | Common Use in Project                    |
| ------------- | ----------------- | ------------------------------ | ---------------------------------------- |
| `Debug`       | Standard Library  | Prints internal structure      | `tracing`, `log::info!("{:?}", item)`    |
| `Clone`       | Standard Library  | Explicit duplication           | Sharing data between threads/closures    |
| `Copy`        | Standard Library  | Implicit bitwise copy          | Small enums (`OrderStatus`), IDs         |
| `Serialize`   | `serde` crate     | Rust struct $\rightarrow$ JSON | HTTP API responses (`ProductResponse`)   |
| `Deserialize` | `serde` crate     | JSON $\rightarrow$ Rust struct | HTTP API payloads (`CreateOrderRequest`) |
| `Error`       | `thiserror` crate | Implements `std::error::Error` | Centralized errors (`AppError`)          |

---

## Rust `Result<T, E>` and `Option<T>` enums

`Result<T, E>` and `Option<T>` are the standard library enums that replace `null` pointers and unchecked exceptions with explicit, compiler-enforced types.  

### 1. Their Definitions in `std`

```rust
// Represents optional absence or presence
pub enum Option<T> {
    Some(T),
    None,
}

// Represents potential success or failure
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

Because both are enums, the compiler prevents accessing the inner value `T` without pattern matching or handling the other variant.

### 2. Common Handling Patterns in `order-engine`

#### Pattern A: The `?` (Try) Operator

Unwraps `Ok(val)` or returns early with `Err(e)`.

```rust
// Without `?` (Manual match boilerplate)
let conn = match get_conn(&pool) {
    Ok(c) => c,
    Err(err) => return Err(err),
};

// With `?` (Crisp early return)
let mut conn = get_conn(&pool)?;
```

#### Pattern B: Bridge `Option` to `Result` via `.ok_or_else()`

Convert an absent query param or missing entity into an explicit error variant.

```rust
// If None, converts to Err(AppError::NotFound)
let user_id = path_param.ok_or_else(|| {
    AppError::NotFound("User ID missing from request path".to_string())
})?;
```

#### Pattern C: Functional Transforms (`.map()` and `.unwrap_or()`)

Transform the inner value without unwrapping manually.

```rust
// 1. .map(): Run closure ONLY if value is present/ok
let desc_slice: Option<&str> = req.description.as_deref();

// 2. .unwrap_or(): Fall back to a default value if None
let page: i64 = query.page.unwrap_or(1);
```

### Quick Reference

| Method            | On `Option<T>`                                    | On `Result<T, E>`                               | Typical Project Use                  |
| ----------------- | ------------------------------------------------- | ----------------------------------------------- | ------------------------------------ |
| `?`               | Returns `None` early                              | Returns `Err(E)` early (with `From` conversion) | Propagating DB errors up the stack   |
| `.map(f)`         | `Some(T)` $\rightarrow$ `Some(f(T))`              | `Ok(T)` $\rightarrow$ `Ok(f(T))`                | Transforming DTOs to DB entities     |
| `.and_then(f)`    | Flattens `Option<Option<U>>`                      | Chains operations returning `Result`            | Sequence of validations              |
| `.ok_or(err)`     | Converts `Option<T>` $\rightarrow$ `Result<T, E>` | —                                               | Treating a missing value as an error |
| `.unwrap_or(def)` | Provides fallback value for `None`                | Provides fallback value for `Err`               | Default query params (`page = 1`)    |

---
