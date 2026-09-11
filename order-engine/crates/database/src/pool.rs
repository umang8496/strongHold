//! ==============================================================================
//! Database Connection Pool Management
//! ==============================================================================
//! Sets up and exposes a thread-safe connection pool using Diesel and r2d2.
//! Also provides helper utilities to acquire connections and translate Diesel
//! errors into domain AppError instances.
//! ==============================================================================

use common::{AppError, AppResult};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

/// Type alias for the r2d2 Connection Manager bound to PostgreSQL.
pub type PgManager = ConnectionManager<PgConnection>;

/// Type alias for the shared connection pool.
/// This pool is cheap to clone (internally wrapped in an Arc) and can be
/// safely shared across Actix worker threads via `web::Data<DbPool>`.
pub type DbPool = Pool<PgManager>;

/// Type alias for a single borrowed connection from the pool.
pub type DbConn = PooledConnection<PgManager>;

/// Initializes a new PostgreSQL connection pool.
///
/// # Arguments
/// * `database_url` - The full PostgreSQL connection string (e.g., "postgres://user:pass@localhost:5432/order_engine")
/// * `max_size` - Maximum number of active connections maintained in the pool
pub fn establish_pool(database_url: &str, max_size: u32) -> Result<DbPool, r2d2::Error> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .max_size(max_size)
        .build(manager)
}

/// Helper function to safely extract a connection from the pool,
/// mapping pool checkout timeouts to `AppError::Internal`.
pub fn get_conn(pool: &DbPool) -> AppResult<DbConn> {
    pool.get().map_err(|err| {
        log::error!("Failed to check out connection from pool: {}", err);
        AppError::Internal("Database connection pool exhausted".to_string())
    })
}

/// Helper extension function to map Diesel Result into AppResult.
/// Translates Diesel's `NotFound` to `AppError::NotFound`, and database
/// query failures to `AppError::DatabaseError`.
pub fn map_diesel_error(err: diesel::result::Error, entity_name: &'static str) -> AppError {
    match err {
        diesel::result::Error::NotFound => {
            AppError::NotFound(format!("{} not found", entity_name))
        }
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            info,
        ) => {
            AppError::Conflict(format!("Duplicate entry: {}", info.message()))
        }
        other => {
            log::error!("Diesel query execution error: {:?}", other);
            AppError::DatabaseError(other.to_string())
        }
    }
}
