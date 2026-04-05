/// Cloudflare API client implementation.
///
/// This module provides an async client for interacting with the Cloudflare API v4.
/// It uses blocking reqwest wrapped in `smol::unblock` to avoid runtime conflicts
/// with iocraft's internal smol executor.
use reqwest::blocking::Client;
use std::sync::Arc;

use super::error::{CloudflareError, CloudflareResult};
use super::models::{ApiResponse, DnsRecord, DnsRecordResponse, ZoneResponse};

/// Internal client state wrapped in Arc for thread-safe sharing.
struct Inner {
    client: Client,
    api_token: String,
    zone_id: String,
    base_url: String,
}

/// Cloudflare API client for DNS record management.
///
/// This client provides methods to list, create, update, and delete DNS records
/// in a Cloudflare zone. It's designed to be cheaply cloneable (via Arc) for use
/// across async tasks.
///
/// # Example
/// ```ignore
/// let client = CloudflareClient::new(api_token, zone_id);
/// let records = client.list_dns_records().await?;
/// ```
pub struct CloudflareClient {
    inner: Arc<Inner>,
}

impl CloudflareClient {
    /// Create a new Cloudflare API client.
    ///
    /// # Arguments
    /// * `api_token` - Cloudflare API token with DNS edit permissions
    /// * `zone_id` - The Cloudflare zone ID for the domain to manage
    pub fn new(api_token: String, zone_id: String) -> Self {
        Self::with_base_url(
            api_token,
            zone_id,
            "https://api.cloudflare.com/client/v4".to_string(),
        )
    }

    /// Create a client with a custom base URL (useful for testing with mock servers).
    pub fn with_base_url(api_token: String, zone_id: String, base_url: String) -> Self {
        Self {
            inner: Arc::new(Inner {
                client: Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .expect("Failed to create HTTP client"),
                api_token,
                zone_id,
                base_url,
            }),
        }
    }

    /// Fetch the zone name for the configured zone ID.
    ///
    /// Returns the zone name if successful, or falls back to the zone ID
    /// if the API request fails.
    pub async fn get_zone_name(&self) -> Result<String, CloudflareError> {
        let inner = self.inner.clone();
        smol::unblock(move || {
            let url = format!("{}/zones/{}", inner.base_url, inner.zone_id);
            let response = inner
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", inner.api_token))
                .header("Content-Type", "application/json")
                .send()
                .map_err(CloudflareError::RequestError)?;

            if !response.status().is_success() {
                return Ok(inner.zone_id.clone());
            }

            let api_response: ApiResponse<ZoneResponse> =
                response.json().map_err(CloudflareError::RequestError)?;

            if let Some(zone) = api_response.result {
                Ok(zone.name)
            } else {
                Ok(inner.zone_id.clone())
            }
        })
        .await
    }

    /// List all DNS records in the configured zone.
    ///
    /// Automatically handles pagination by fetching all pages of records.
    pub async fn list_dns_records(&self) -> CloudflareResult<Vec<DnsRecord>> {
        let inner = self.inner.clone();
        smol::unblock(move || {
            let mut all_records = Vec::new();
            let mut page = 1;
            let per_page = 100;

            loop {
                let url = format!(
                    "{}/zones/{}/dns_records?page={}&per_page={}",
                    inner.base_url, inner.zone_id, page, per_page
                );

                let response = inner
                    .client
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", inner.api_token))
                    .header("Content-Type", "application/json")
                    .send()
                    .map_err(CloudflareError::RequestError)?;

                if !response.status().is_success() {
                    let status = response.status().as_u16();
                    let body = response.text().unwrap_or_default();
                    return Err(CloudflareError::HttpError { status, body });
                }

                let api_response: ApiResponse<Vec<DnsRecordResponse>> =
                    response.json().map_err(CloudflareError::RequestError)?;

                if !api_response.success {
                    let errors: Vec<String> = api_response
                        .errors
                        .iter()
                        .map(|e| format!("{} (code: {})", e.message, e.code))
                        .collect();
                    return Err(CloudflareError::ApiErrors(errors));
                }

                let records = api_response.result.unwrap_or_default();
                let fetched_count = records.len();
                if fetched_count == 0 {
                    break;
                }

                all_records.extend(records.into_iter().map(|r| DnsRecord {
                    id: Some(r.id),
                    record_type: r.record_type,
                    name: r.name,
                    content: r.content,
                    ttl: Some(r.ttl),
                    proxied: Some(r.proxied),
                    comment: r.comment,
                }));

                if fetched_count < per_page {
                    break;
                }
                page += 1;
            }

            Ok(all_records)
        })
        .await
    }

    /// Create a new DNS record in the configured zone.
    pub async fn create_dns_record(&self, record: &DnsRecord) -> CloudflareResult<DnsRecord> {
        let inner = self.inner.clone();
        let record = record.clone();

        smol::unblock(move || {
            let response = inner
                .client
                .post(format!(
                    "{}/zones/{}/dns_records",
                    inner.base_url, inner.zone_id
                ))
                .header("Authorization", format!("Bearer {}", inner.api_token))
                .header("Content-Type", "application/json")
                .json(&record)
                .send()
                .map_err(CloudflareError::RequestError)?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let body = response.text().unwrap_or_default();
                return Err(CloudflareError::HttpError { status, body });
            }

            let api_response: ApiResponse<DnsRecordResponse> =
                response.json().map_err(CloudflareError::RequestError)?;

            if !api_response.success {
                let errors: Vec<String> = api_response
                    .errors
                    .iter()
                    .map(|e| format!("{} (code: {})", e.message, e.code))
                    .collect();
                return Err(CloudflareError::ApiErrors(errors));
            }

            let result = api_response.result.ok_or(CloudflareError::NoResult)?;

            Ok(DnsRecord {
                id: Some(result.id),
                record_type: result.record_type,
                name: result.name,
                content: result.content,
                ttl: Some(result.ttl),
                proxied: Some(result.proxied),
                comment: result.comment,
            })
        })
        .await
    }

    /// Update an existing DNS record.
    ///
    /// The record must have an `id` field set, otherwise returns `MissingRecordId`.
    pub async fn update_dns_record(&self, record: &DnsRecord) -> CloudflareResult<DnsRecord> {
        let inner = self.inner.clone();
        let record = record.clone();

        smol::unblock(move || {
            let record_id = record.id.as_ref().ok_or(CloudflareError::MissingRecordId)?;

            let response = inner
                .client
                .put(format!(
                    "{}/zones/{}/dns_records/{}",
                    inner.base_url, inner.zone_id, record_id
                ))
                .header("Authorization", format!("Bearer {}", inner.api_token))
                .header("Content-Type", "application/json")
                .json(&record)
                .send()
                .map_err(CloudflareError::RequestError)?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let body = response.text().unwrap_or_default();
                return Err(CloudflareError::HttpError { status, body });
            }

            let api_response: ApiResponse<DnsRecordResponse> =
                response.json().map_err(CloudflareError::RequestError)?;

            if !api_response.success {
                let errors: Vec<String> = api_response
                    .errors
                    .iter()
                    .map(|e| format!("{} (code: {})", e.message, e.code))
                    .collect();
                return Err(CloudflareError::ApiErrors(errors));
            }

            let result = api_response.result.ok_or(CloudflareError::NoResult)?;

            Ok(DnsRecord {
                id: Some(result.id),
                record_type: result.record_type,
                name: result.name,
                content: result.content,
                ttl: Some(result.ttl),
                proxied: Some(result.proxied),
                comment: result.comment,
            })
        })
        .await
    }

    /// Delete a DNS record by its ID.
    pub async fn delete_dns_record(&self, record_id: &str) -> CloudflareResult<()> {
        let inner = self.inner.clone();
        let record_id = record_id.to_string();

        smol::unblock(move || {
            let response = inner
                .client
                .delete(format!(
                    "{}/zones/{}/dns_records/{}",
                    inner.base_url, inner.zone_id, record_id
                ))
                .header("Authorization", format!("Bearer {}", inner.api_token))
                .header("Content-Type", "application/json")
                .send()
                .map_err(CloudflareError::RequestError)?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let body = response.text().unwrap_or_default();
                return Err(CloudflareError::HttpError { status, body });
            }

            let api_response: ApiResponse<serde_json::Value> =
                response.json().map_err(CloudflareError::RequestError)?;

            if !api_response.success {
                let errors: Vec<String> = api_response
                    .errors
                    .iter()
                    .map(|e| format!("{} (code: {})", e.message, e.code))
                    .collect();
                return Err(CloudflareError::ApiErrors(errors));
            }

            Ok(())
        })
        .await
    }
}
