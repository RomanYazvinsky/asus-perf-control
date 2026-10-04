// Rust library root for ASUS Linux Control Tool
// Hardware access shared by the desktop UI (rust_ui)

pub mod api;
pub mod services;

// Re-export main API types and functions
pub use api::*;
