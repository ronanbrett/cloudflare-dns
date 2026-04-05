/// Cloudflare DNS Manager - Library interface.
///
/// This library provides programmatic access to the application's core modules,
/// primarily used for integration testing.
pub mod api;
pub mod config;
pub mod tasks;
pub mod ui;
pub mod utils;

// Re-export commonly used types at the crate root for convenience
pub use api::{CloudflareClient, DnsCache, DnsRecord};
pub use config::Config;
pub use tasks::{DeleteParams, SubmitParams, delete_task, fetch_all, refresh_task, submit_task};
pub use ui::{AppState, AppView, run_app};
pub use utils::{extract_unique_ips, format_records, format_selector, strip_domain_suffix};
