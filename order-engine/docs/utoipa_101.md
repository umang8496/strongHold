<!-- markdownlint-disable MD001 -->
<!-- markdownlint-disable MD024 -->
<!-- markdownlint-disable MD025 -->
<!-- markdownlint-disable MD026 -->
<!-- markdownlint-disable MD040 -->
<!-- markdownlint-disable MD056 -->
<!-- markdownlint-disable MD060 -->

# Engineering Reference Manual: OpenAPI 3.0 Documentation with Utoipa

**Utoipa** (pronounced *u-toy-pa*, derived from *µ-Type-OpenAPI*) is a compile-time OpenAPI v3.0 code generator designed specifically for the Rust web ecosystem.  
It generates compliant OpenAPI JSON/YAML specifications and serves embedded interactive documentation (via Swagger UI, Redoc, or RapiDoc) directly from the compiled binary with zero runtime overhead.

## 1. Architectural Overview & Value Proposition

### How Utoipa Compares to Runtime Documenters (Springdoc, Fastify, FastAPI)

| Criterion | Springdoc (Java) / FastAPI (Python) | Utoipa (Rust) |
| --- | --- | --- |
| **Introspection Mechanism** | Runtime Reflection / Dynamic Type Inspection | Rust Procedural Macros (`proc_macro`) evaluated strictly during `cargo build` |
| **Startup Overhead** | High (builds OpenAPI AST via classpath scanning or type analysis on process boot) | Zero (the AST is generated and baked into binary data segments at compile time) |
| **Type Integrity Guarantees** | Runtime drift possible if types change via dynamic dispatch or unmapped DTOs | **Compile-Time Rejection**: compilation fails if a referenced struct lacks `ToSchema` or if a route parameter does not match the handler signature |
| **Memory Footprint** | Retains heavy metadata models in heap memory | OpenAPI specification lives as static text/structs with zero-copy deserialization |
| **Binary Portability** | Requires external UI asset distributions or dynamic webjars | Embeds static HTML/JS/CSS assets for Swagger UI directly into the compiled ELF/Mach-O binary |

### Macro Pipeline and Mechanics

```text
                  ┌───────────────────────────────┐
                  │    Rust Source Code AST       │
                  └──────────────┬────────────────┘
                                 │
                 ┌───────────────┴───────────────┐
                 │                               │
                 ▼                               ▼
      #[derive(ToSchema)]                #[utoipa::path(...)]
     Evaluates DTO shape,               Analyzes HTTP verb, path,
    docstrings, & validations           params, bodies, & status codes
                 │                               │
                 └───────────────┬───────────────┘
                                 │
                                 ▼
                     #[derive(OpenApi)]
           Aggregates Paths + Component Schemas
                                 │
                                 ▼
                    Compile-Time Emission:
             utoipa::openapi::OpenApi Struct
                                 │
            ┌────────────────────┴────────────────────┐
            ▼                                         ▼
   /api-docs/openapi.json                     /swagger-ui/
   (Raw JSON Spec output)             (utoipa-swagger-ui handler)
```

1. **`#[derive(ToSchema)]`**: Inspects struct fields, types, documentation comments, and helper attributes (`#[schema(...)]`) to generate the OpenAPI `components/schemas` definitions.
2. **`#[utoipa::path(...)]`**: Attaches to route handlers.  
  It extracts the HTTP method, endpoint path, URL parameters, request payloads, response schemas, and error types.
3. **`#[derive(OpenApi)]`**: Acts as the central composition manifest.  
  It aggregates all declared paths, component schemas, security schemes, and global tags into a single immutable `OpenApi` struct.
4. **Binary Embedding**: If using `utoipa-swagger-ui`, UI assets are compressed and bundled at compile time using `include_bytes!` under the hood, eliminating external file dependencies.

## 2. Dependency Architecture & Workspace Setup

To avoid dependency drift in a multi-crate modular monolith, declare `utoipa` and its UI companion in the root `Cargo.toml`.

### Root `Cargo.toml`

```toml
[workspace.dependencies]
# Core compile-time OpenAPI generator
utoipa = { version = "5.3", features = ["actix_extras", "chrono", "uuid"] }

# Self-contained Swagger UI wrapper for Actix Web
utoipa-swagger-ui = { version = "9.0", features = ["actix-web"] }
```

### Domain Crate Manifest (`crates/catalog/Cargo.toml`)

Domain crates require only the macro definitions and type integrations:

```toml
[dependencies]
utoipa = { workspace = true }
serde = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
```

### Composition Root Manifest (`crates/app/Cargo.toml`)

The application crate requires both the generator and the UI renderer:

```toml
[dependencies]
utoipa = { workspace = true }
utoipa-swagger-ui = { workspace = true }
catalog = { path = "../catalog" }
inventory = { path = "../inventory" }
orders = { path = "../orders" }
common = { path = "../common" }
```

## 3. Modeling Schemas: `ToSchema` Implementation Patterns

### Standard Model Derivation with Attribute Overrides

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a sellable physical product in the catalog.
/// Markdown written in docstrings is automatically transferred to OpenAPI descriptions.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = catalog::ProductResponse)] // Explicit schema naming to avoid collisions
pub struct ProductResponse {
    /// Universally unique identifier of the catalog item.
    #[schema(example = "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8")]
    pub id: Uuid,

    /// Unique Stock Keeping Unit (alphanumeric identifier).
    #[schema(example = "KB-MECH-01", max_length = 64, min_length = 3)]
    pub sku: String,

    /// Display title for retail presentation.
    #[schema(example = "Mechanical Keyboard V2", max_length = 255)]
    pub title: String,

    /// Extended markdown-compatible product description.
    #[schema(example = "Hot-swappable mechanical keyboard featuring tactile switches.")]
    pub description: Option<String>,

    /// Retail cost stored in the smallest fractional unit (cents) to avoid precision loss.
    #[schema(example = 12900, minimum = 0)]
    pub price_cents: i64,

    /// Flag designating whether the product is queryable by standard customers.
    #[schema(default = true)]
    pub is_active: bool,

    /// Creation timestamp in RFC 3339 format.
    #[schema(value_type = String, format = DateTime, example = "2026-03-31T08:30:00Z")]
    pub created_at: DateTime<Utc>,
}
```

### Enums & State Representations

Utoipa supports documenting string-backed, integer-backed, or internally tagged enums:

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The processing state of an order envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// Initial state prior to stock allocation.
    Draft,
    /// Inventory items have been reserved atomically.
    Reserved,
    /// Gateway settlement confirmed.
    Paid,
    /// Fulfilled and shipped to destination.
    Completed,
    /// Voided; reserved stock has been released.
    Canceled,
}
```

### Inlined Child Schemas and Nesting

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrderItemRequest {
    #[schema(example = "b2c3d4e5-f6a7-8b9c-0d1e-2f3a4b5c6d7e")]
    pub product_id: Uuid,
    
    #[schema(example = 2, minimum = 1, maximum = 100)]
    pub quantity: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrderRequest {
    #[schema(example = "e5e6e7e8-f9a0-1b2c-3d4e-5f6a7b8c9d0e")]
    pub customer_id: Uuid,

    /// Array of items associated with this checkout operation.
    pub items: Vec<CreateOrderItemRequest>,
}
```

## 4. Documenting Route Handlers: `#[utoipa::path(...)]`

The `#[utoipa::path]` macro must sit immediately above the route function or above Actix method macros (`#[post(...)]`, `#[get(...)]`).

### Complete Endpoint Declaration

```rust
use actix_web::{get, web, HttpResponse};
use common::{AppResult, ErrorResponseBody};
use database::DbPool;
use uuid::Uuid;

use crate::dto::ProductResponse;
use crate::service;

#[utoipa::path(
    get,
    path = "/api/v1/products/{id}",
    tag = "Catalog",
    operation_id = "catalog_get_product_by_id",
    summary = "Fetch product by ID",
    description = "Queries the database for an active product by its primary UUID key. Inactive or soft-deleted items return 404.",
    params(
        ("id" = Uuid, Path, description = "Target product unique identifier", example = "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8")
    ),
    responses(
        (status = 200, description = "Product successfully retrieved", body = ProductResponse),
        (status = 404, description = "Product not found or soft-deleted", body = ErrorResponseBody),
        (status = 500, description = "Internal database failure", body = ErrorResponseBody)
    )
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
```

### Parameter Categories Supported

* **Path Parameters**: Declared directly in the `params(...)` block:

```rust
("id" = Uuid, Path, description = "Resource identifier")
```

* **Query Parameters (Derived via Struct)**:

```rust
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationFilter {
    /// Page number index (1-based)
    pub page: Option<i64>,
    /// Total rows per slice
    pub page_size: Option<i64>,
}

// Inside #[utoipa::path]:
// params(PaginationFilter)
```

* **Headers**:

```rust
("X-Idempotency-Key" = String, Header, description = "Unique transaction token", example = "req_10928374")
```

## 5. Security Schemes & Authentication Handlers

To declare secured routes, configure global `SecurityScheme` primitives in the root manifest, then link them to routes using the `security(...)` attribute.

### Registering Security Definitions in the OpenApi Struct

```rust
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::Modify;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().expect("Components must exist");
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("Enter your signed JSON Web Token (without 'Bearer ' prefix)"))
                    .build(),
            ),
        );
    }
}
```

### Attaching Security to Route Handlers

```rust
#[utoipa::path(
    post,
    path = "/api/v1/orders",
    tag = "Orders",
    security(
        ("bearer_auth" = [])
    ),
    request_body = CreateOrderRequest,
    responses(
        (status = 201, description = "Order checked out successfully", body = OrderDetailResponse),
        (status = 401, description = "Missing or invalid authorization token", body = ErrorResponseBody),
        (status = 409, description = "Insufficient inventory to reserve", body = ErrorResponseBody)
    )
)]
pub async fn create_order_handler(...) -> AppResult<HttpResponse> { ... }
```

## 6. The Composition Root: `crates/app/src/main.rs`

The composition root aggregates all modules, paths, schemas, and serves Swagger UI.

```rust
use actix_web::{middleware::Logger, web, App, HttpServer};
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Import paths and models from workspace member crates
use catalog::controller::{create_product_handler, get_product_handler, list_products_handler};
use catalog::dto::{CreateProductRequest, ProductResponse, UpdateProductRequest};
use common::ErrorResponseBody;

#[derive(OpenApi)]
#[openapi(
    paths(
        catalog::controller::create_product_handler,
        catalog::controller::get_product_handler,
        catalog::controller::list_products_handler,
    ),
    components(
        schemas(
            CreateProductRequest,
            UpdateProductRequest,
            ProductResponse,
            ErrorResponseBody
        )
    ),
    tags(
        (name = "Catalog", description = "Product and category management"),
        (name = "Inventory", description = "Stock tracking and atomic allocations"),
        (name = "Orders", description = "Order state machine and checkout operations")
    ),
    info(
        title = "Order Engine Modular Monolith API",
        version = "1.0.0",
        description = "Production-grade enterprise modular monolith written in Rust, Actix-web, and Diesel.",
        contact(
            name = "Platform Engineering Team",
            email = "engineering@orderengine.internal"
        ),
        license(
            name = "Proprietary",
            url = "https://orderengine.internal/license"
        )
    )
)]
pub struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let server_port: u16 = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid u16");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be configured");
    let pool = database::establish_pool(&database_url, 10).expect("Failed to initialize pool");
    let pool_data = web::Data::new(pool);

    log::info!("Starting Actix HTTP Server on port {}", server_port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(pool_data.clone())
            // Mount OpenAPI Spec JSON and Swagger UI
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .configure(catalog::configure_routes)
    })
    .bind(("127.0.0.1", server_port))?
    .run()
    .await
}
```

## 7. CI/CD Static Generation & Spec Extraction

Running an active server process just to scrape the OpenAPI document is fragile in CI/CD pipelines.  
`Utoipa` allows you to extract the spec as a compile-time target via an isolated binary or build script.

### Creating a Spec Extractor Binary

Create `crates/app/src/bin/generate_openapi.rs`:

```rust
use std::fs::File;
use std::io::Write;
use utoipa::OpenApi;

// Import ApiDoc from main application library or crate root
use app::ApiDoc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec_json = ApiDoc::openapi().to_pretty_json()?;
    
    let target_path = "openapi.json";
    let mut file = File::create(target_path)?;
    file.write_all(spec_json.as_bytes())?;

    println!("Successfully exported OpenAPI v3.0 spec to {}", target_path);
    Ok(())
}
```

Add the target to `crates/app/Cargo.toml`:

```toml
[[bin]]
name = "generate-openapi"
path = "src/bin/generate_openapi.rs"
```

You can now run:

```bash
cargo run -p app --bin generate-openapi
```

This writes `openapi.json` directly to disk without requiring a running PostgreSQL instance, active network ports, or runtime environment variables.  
This JSON artifact can then be fed into contract validation suites, Spectral linters, or SDK generators (like `openapi-generator-cli`).

## 8. Common Pitfalls, Diagnostic Patterns, and Best Practices

### 1. The "Orphaned Schema" Compilation Trap

* **Symptom**: You added `#[derive(ToSchema)]` to a struct, but it does not appear in the generated Swagger UI or raw JSON.
* **Root Cause**: Deriving `ToSchema` only produces the `impl utoipa::ToSchema for StructName` code.  
  It does not automatically register the struct with the central OpenAPI document.
* **Remedy**: You must explicitly register the struct inside the `components(schemas(...))` array within the `#[derive(OpenApi)]` macro on `ApiDoc`.

### 2. Visibility and Encapsulation

* **Symptom**: Compiler error `cannot find type 'MyType' in this scope` inside `#[openapi(...)]`.
* **Root Cause**: `utoipa::path` and `utoipa::OpenApi` macros generate code that references structs using their local path scope.  
  If your handlers or DTOs are private (`pub(crate)` or unannotated), the composition root in `crates/app` cannot access their types.
* **Remedy**: Ensure that all controllers and DTOs intended for documentation are marked with `pub`.

### 3. Chrono `DateTime` and UUID Formatting Anomalies

* **Symptom**: `DateTime<Utc>` fields appear as empty objects (`{}`) or arbitrary integers instead of ISO 8601 strings.
* **Root Cause**: The workspace `utoipa` crate was imported without standard scalar integration features.
* **Remedy**: Always activate the native format features in your dependency declaration:

```toml
utoipa = { version = "5.3", features = ["chrono", "uuid"] }
```

### 4. Docstring Leaks

* Any Rust triple-slash (`///`) docstring attached to structs, fields, or functions is converted directly into user-facing OpenAPI descriptions.  
  Keep internal development notes in double-slash comments (`//`) so implementation details are not published to API consumers.

---
