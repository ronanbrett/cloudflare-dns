/// Utility functions for formatting and processing.
///
/// This module contains pure functions for data transformation and display formatting.
pub mod formatting;

pub use formatting::{extract_unique_ips, format_records, format_selector, strip_domain_suffix};
