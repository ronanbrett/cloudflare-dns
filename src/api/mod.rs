/// Cloudflare API module.
///
/// This module provides the client, models, error types, and caching for interacting
/// with the Cloudflare API v4.
pub mod cache;
pub mod client;
pub mod error;
pub mod models;

// Re-export commonly used types for library consumers
#[allow(unused_imports)]
pub use cache::DnsCache;
pub use client::CloudflareClient;
#[allow(unused_imports)]
pub use error::{CloudflareError, CloudflareResult};
pub use models::DnsRecord;
