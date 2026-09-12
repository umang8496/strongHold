//! ==============================================================================
//! Catalog Domain Crate
//! ==============================================================================
//! Owns products and categories, product search/filtering, and pricing rules.
//! ==============================================================================

pub mod controller;
pub mod dto;
pub mod models;
pub mod repository;
pub mod service;

// Re-export the route configurator for the root binary to mount
pub use controller::configure_routes;
