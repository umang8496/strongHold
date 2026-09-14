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
- [What is `HttpServer::new(move || ...)`?](#what-is-httpservernewmove--)
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

## What is `HttpServer::new(move || ...)`?

In standard multi-threaded web servers (like Spring Boot with embedded Tomcat or Go's `net/http`), server initialization typically accepts a pre-constructed router or application object.

`Actix-web` takes a different route: `HttpServer::new` takes a **closure that acts as an application factory**:

```rust
HttpServer::new(move || {
    App::new()
        .wrap(Logger::default())
        .app_data(pool_data.clone())
        .configure(catalog::configure_routes)
})
```

To understand why Actix-web requires this `closure`, what `move` achieves under the borrow checker, and how state travels across operating system threads,  
we must look at Actix-web’s underlying thread and runtime architecture.  

### 1. Actix-web Runtime Architecture: Multi-Reactor Model

Unlike typical web frameworks that use a single shared routing table protected by mutexes, Actix-web implements a **multi-reactor thread-per-core design**.

```text
                           [ Main OS Thread ]
                                   │
                     Creates TCP Listener Socket
                     SO_REUSEPORT / Socket Handoff
                                   │
               ┌───────────────────┼───────────────────┐
               ▼                   ▼                   ▼
      [ Worker Thread 0 ]  [ Worker Thread 1 ]  [ Worker Thread N-1 ]
      ┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
      │ Tokio Reactor/RT │ │ Tokio Reactor/RT │ │ Tokio Reactor/RT │
      │ Local Current-   │ │ Local Current-   │ │ Local Current-   │
      │ Thread Runtime   │ │ Thread Runtime   │ │ Thread Runtime   │
      │                  │ │                  │ │                  │
      │   App Instance   │ │   App Instance   │ │   App Instance   │
      │   - Router       │ │   - Router       │ │   - Router       │
      │   - Middlewares  │ │   - Middlewares  │ │   - Middlewares  │
      │   - AppState Map │ │   - AppState Map │ │   - AppState Map │
      └──────────────────┘ └──────────────────┘ └──────────────────┘
               ▲                   ▲                   ▲
               │                   │                   │
         Incoming TCP        Incoming TCP        Incoming TCP
          Connections         Connections         Connections
```

#### The Worker Process Lifecycle

1. When you boot the application, Actix detects the available hardware parallelism (e.g., 8 logical cores = 8 workers).
2. It binds a single root listening TCP socket (`127.0.0.1:8080`).
3. It spawns $N$ independent operating system threads via `std::thread::spawn`.
4. **Crucial detail:** Each worker thread runs its own dedicated, single-threaded **current-thread Tokio runtime** and event loop (reactor).
5. Incoming client connections are accepted and distributed to worker threads using round-robin socket handoffs.

### 2. Why a Closure? The Factory Pattern vs. Shared Application

Consider what would happen if `HttpServer::new` took an instantiated struct rather than a closure:

```rust
// HYPOTHETICAL — THIS DOES NOT EXIST IN ACTIX:
let app = App::new().service(...);
HttpServer::new(app); 
```

If one global `app` existed:

- **Synchronization Bottleneck:**  
  Every incoming HTTP request across all 8 cores would need to access the router, middleware chains, and resource tables through synchronized locks (`Arc<RwLock<Router>>`).  
  Under heavy concurrent load (100k+ req/sec), thread contention on the routing table would stall the CPU cores.
- **Loss of Thread Locality:**  
  Middlewares, state handlers, and encoders could not leverage thread-local caching.  
  Everything would require cross-thread synchronization primitives.

#### The Solution: The Factory Closure

Instead of taking an instance, `HttpServer::new` takes a **Factory**:

```rust
pub struct HttpServer<F, I, S, B>
where
    F: Fn() -> I + Send + Clone + 'static,
    I: IntoServiceFactory<S, Request>,
    S: ServiceFactory<Request, Config = AppConfig>,
    ...
```

Notice the trait bound on `F`:

$$\text{Factory } F: \text{Fn}() \longrightarrow I + \text{Send} + \text{Clone} + \text{'static}$$

- **`Fn()`**: It is callable without mutating internal state, and can be invoked repeatedly.
- **`Clone`**: Actix clones the factory function across thread boundaries.
- **`Send`**: The factory can be safely transferred to newly spawned OS threads.
- **`'static`**: The factory does not borrow data that might be deallocated when `main()` completes.

When Actix spawns 8 worker threads, **each thread calls your closure once**.

Each worker thread constructs its own independent, isolated `App` instance directly inside its own thread-local memory.  
Route matching, middleware pipelines, and state extraction execute with **zero lock contention** between CPU cores.

### 3. Deconstructing the Syntax: `move || { ... }`

The syntax consists of two distinct components: the closure definition (`||`) and the capture mode (`move`).

#### 1. The Pipe Syntax: `||`

`||` denotes an anonymous function taking zero arguments. It defines the blueprint of how to build an `App`.

#### 2. The `move` Keyword: Value Capture by Value vs. Reference

In Rust, closures capture variables from their enclosing scope in one of three ways:

- By reference: `&T`
- By mutable reference: `&mut T`
- By value (ownership transfer): `T`

Without the `move` keyword, closures default to capturing variables by reference:

```rust
// Without `move`:
let pool_data = web::Data::new(pool);

HttpServer::new(|| {
    // The closure attempts to borrow `&pool_data` from the outer scope!
    App::new().app_data(pool_data.clone())
})
```

The Rust compiler rejects this with a clear lifetime error:

```text
error[E0373]: closure may outlive the current function, but it borrows `pool_data`,
              which is owned by the current function
  --> src/main.rs:40:21
   |
40 |     HttpServer::new(|| {
   |                     ^^ may outlive borrowed value `pool_data`
41 |         App::new().app_data(pool_data.clone())
   |                             --------- `pool_data` is borrowed here
```

#### Why the Borrow Checker Enforces This

1. `pool_data` is allocated on the stack frame of `fn main()`.
2. `HttpServer::new(...)` launches OS threads via `std::thread::spawn`.
3. Worker threads run indefinitely in the background. The compiler cannot statically guarantee that `main()` will never exit before the worker threads finish running.
4. If `main()` were to terminate or drop its stack frame, a borrowed reference `&pool_data` would point to deallocated memory—causing a **use-after-free** segment violation.

By prepending `move`, you force the closure to **relinquish references and seize full ownership** of captured variables:

```rust
HttpServer::new(move || {
    // `pool_data` has been MOVED into the closure's environment block.
    // It now lives as long as the closure lives.
    App::new().app_data(pool_data.clone())
})
```

### 4. The Two-Tier Cloning Pipeline

A common point of confusion: **Why do we call `.clone()` twice on the pool?**

Look at the structure in `main.rs`:

```rust
// Tier 1: Outside the server definition
let pool_data = web::Data::new(pool.clone()); 

HttpServer::new(move || {
    App::new()
        // Tier 2: Inside the closure body
        .app_data(pool_data.clone()) 
})
```

Tracing this step-by-step through execution reveals what happens at both tiers:

```text
[ Step 1: Main Thread Initial State ]
  pool_data: web::Data<DbPool> (Arc strong_count = 1)
       │
       ▼
[ Step 2: `HttpServer::new(move || ...)` ]
  `move` transfers `pool_data` into the closure environment struct.
  The closure now owns `pool_data`.
       │
       ▼
[ Step 3: Actix spawns Worker Threads ]
  Actix clones the closure once for each OS worker thread.
       │
       ├────────────────────────┬────────────────────────┐
       ▼                        ▼                        ▼
[ Worker Thread 0 ]      [ Worker Thread 1 ]      [ Worker Thread N-1 ]
Closure executes:        Closure executes:        Closure executes:
`pool_data.clone()`      `pool_data.clone()`      `pool_data.clone()`
       │                        │                        │
       ▼                        ▼                        ▼
Increment Arc count      Increment Arc count      Increment Arc count
(strong_count = 2)       (strong_count = 3)       (strong_count = N+1)
       │                        │                        │
Stored in Worker 0's     Stored in Worker 1's     Stored in Worker N-1's
Local App State Map      Local App State Map      Local App State Map
```

#### Why Call `.clone()` Inside the Closure?

The closure is an `Fn()` factory invoked $N$ times.

- If you wrote `.app_data(pool_data)` (without `.clone()`), the closure would attempt to consume and move `pool_data` during its **first** run on Worker Thread 0.
- When Worker Thread 1 ran the closure next, `pool_data` would no longer exist—it would be a use-of-moved-value compiler error.
- Calling `pool_data.clone()` inside the closure increments the atomic reference counter (`Arc`), giving each worker thread's local `App` state its own valid handle to the same underlying connection pool.

### 5. Architectural Comparison: How Frameworks Solve Concurrency

| Framework / Ecosystem              | Threading Model                                                               | Application State Distribution                                                                                              | Cost / Trade-off                                                                                                                |
| ---------------------------------- | ----------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| **Spring Boot (Embedded Tomcat)**  | One thread per request (Platform Threads) or Virtual Threads (Project Loom).  | Global singleton beans managed by the `ApplicationContext` heap map.                                                        | Requires thread-safe shared singletons; concurrent access relies on synchronized blocks or atomic fields.                       |
| **Go (`net/http`)**                | Multiplexed lightweight goroutines over $M:N$ scheduler.                      | Shared pointer structures passed down through closures or request `context.Context`.                                        | Lightweight instantiation, but shared state structures must manage their own mutexes (`sync.RWMutex`).                          |
| **Actix-web (Rust)**               | Thread-per-core multi-reactor using system threads + local Tokio runtimes.    | **Zero-sharing by default.** Independent `App` instances per thread; shared state explicitly requires `web::Data` (`Arc`).  | Startup requires factory closures (`move ||`), but guarantees zero lock contention on routers and middlewares during execution. |

#### Mental Model Takeaway

When writing:

```rust
HttpServer::new(move || {
    App::new()
        .app_data(pool_data.clone())
        .configure(routes)
})
```

You are giving Actix an **assembly recipe**:

1. **`HttpServer::new(...)`**: Spawns an isolated worker thread on every CPU core.
2. **`move`**: Transfers ownership of the ingredients (`pool_data`) from `main`'s stack frame into the recipe box so it remains valid indefinitely.
3. **`|| { ... }`**: The recipe itself. Each worker thread runs this block independently to construct its own dedicated router, middleware stack, and local state map.
4. **`pool_data.clone()`**: Hands that worker's local state an atomic pointer to the shared connection pool, keeping network connections pooled while routing remains lock-free.

---
