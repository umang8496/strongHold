//! ==============================================================================
//! Common Shared Library
//! ==============================================================================
//! Contains cross-cutting domain-agnostic utilities, shared DTOs,
//! and error definitions used across all workspace crates.
//! ==============================================================================

pub mod error;

// Re-export core error types at the root of the crate for ergonomics.
// Callers can do `use common::{AppError, AppResult};` directly.
pub use error::{AppError, AppResult};
