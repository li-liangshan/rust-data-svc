pub mod demo;
pub mod health;
pub mod metrics;

pub use demo::demo_router;
pub use health::health_router;
pub use metrics::metrics_router;
