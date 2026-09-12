pub mod controller;
pub mod dto;
pub mod models;
pub mod repository;
pub mod service;

pub use controller::configure_routes;
pub use service::{InventoryPort, LocalInventoryService};
