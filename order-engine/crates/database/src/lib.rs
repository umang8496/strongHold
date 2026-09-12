//! ==============================================================================
//! Database Infrastructure Crate
//! ==============================================================================
//! Owns connection pooling, Diesel query helpers, and the generated schema.
//! ==============================================================================

pub mod pool;
pub mod schema;

// Re-export pool types directly at crate root
pub use pool::{establish_pool, get_conn, map_diesel_error, DbConn, DbPool, PgManager};
