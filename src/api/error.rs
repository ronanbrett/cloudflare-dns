/// Custom error types for Cloudflare API operations.
///
/// This module provides domain-specific error types that enable better error handling
/// and more user-friendly error messages compared to generic `anyhow::Error`.
use thiserror::Error;

/// Errors that can occur during Cloudflare API operations.
#[derive(Debug, Error)]
pub enum CloudflareError {
    /// API returned an error response with specific error codes and messages.
    #[allow(dead_code)]
    #[error("API error: {message} (code: {code})")]
    ApiError {
        /// The error code from Cloudflare API
        code: i64,
        /// The error message from Cloudflare API
        message: String,
    },

    /// Multiple errors returned from Cloudflare API.
    #[error("API errors: {}", .0.join(", "))]
    ApiErrors(Vec<String>),

    /// HTTP request failed (network errors, timeouts, etc.).
    #[error("HTTP request failed with status {status}: {body}")]
    HttpError {
        /// HTTP status code
        status: u16,
        /// Response body
        body: String,
    },

    /// Failed to parse API response.
    #[error("Failed to parse API response: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Failed to send HTTP request.
    #[error("Failed to send request: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Record has no ID when trying to update.
    #[error("Cannot update record: no ID provided")]
    MissingRecordId,

    /// No result returned from API.
    #[error("No result returned from API")]
    NoResult,
}

/// Result type alias for Cloudflare API operations.
pub type CloudflareResult<T> = Result<T, CloudflareError>;
