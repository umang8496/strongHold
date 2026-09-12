//! ==============================================================================
//! Application Entry Point (Composition Root)
//! ==============================================================================
//! Reads configuration, sets up observability, initializes the database connection
//! pool, binds HTTP routes from domain modules, and starts the Actix HTTP server.
//! ==============================================================================

use actix_web::{middleware::Logger, web, App, HttpServer};
use inventory::{InventoryPort, LocalInventoryService};
use std::env;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // 2. Initialize structured logging from RUST_LOG environment variable
    // Defaults to 'info' level if RUST_LOG is not explicitly set.
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Bootstrapping Order Engine Modular Monolith...");

    // 3. Extract runtime configuration
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set in .env");

    let server_port: u16 = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid 16-bit unsigned integer");

    // 4. Initialize the PostgreSQL connection pool (max 10 connections)
    log::info!("Connecting to PostgreSQL connection pool...");
    let pool = database::establish_pool(&database_url, 10)
        .expect("Failed to create PostgreSQL connection pool");

    log::info!("Database connection pool successfully initialized.");

    // 5. Wrap the pool in Actix's shared web::Data container
    let pool_data = web::Data::new(pool.clone());

    // Construct the concrete inventory service and bind it to the public port trait
    let inventory_service: Arc<dyn InventoryPort> = Arc::new(LocalInventoryService::new(pool.clone()));
    let inventory_port_data = web::Data::new(inventory_service);

    log::info!("Starting HTTP server on 127.0.0.1:{}...", server_port);

    // 6. Launch the Actix HTTP Server
    HttpServer::new(move || {
        App::new()
            // Injects HTTP request/response logging middleware
            .wrap(Logger::default())
            // Register shared DB pool for extractors
            .app_data(pool_data.clone())
            // Mount domain route configurators
            // Register shared DB pool for extractors
            .app_data(inventory_port_data.clone())            
            .configure(catalog::configure_routes)
            .configure(inventory::configure_routes)
            .configure(orders::configure_routes)
    })
    .bind(("127.0.0.1", server_port))?
    .run()
    .await
}
