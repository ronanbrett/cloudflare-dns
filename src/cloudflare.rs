use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: Option<i64>,
    pub proxied: Option<bool>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    pub success: bool,
    pub errors: Vec<ApiError>,
    pub messages: Vec<String>,
    pub result: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DnsRecordResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: i64,
    pub proxied: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ZoneResponse {
    pub id: String,
    pub name: String,
}

struct Inner {
    client: Client,
    api_token: String,
    zone_id: String,
    base_url: String,
}

pub struct CloudflareClient {
    inner: Arc<Inner>,
}

impl CloudflareClient {
    pub fn new(api_token: String, zone_id: String) -> Self {
        Self {
            inner: Arc::new(Inner {
                client: Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .expect("Failed to create HTTP client"),
                api_token,
                zone_id,
                base_url: "https://api.cloudflare.com/client/v4".to_string(),
            }),
        }
    }

    pub async fn get_zone_name(&self) -> Result<String> {
        let inner = self.inner.clone();
        smol::unblock(move || {
            let url = format!("{}/zones/{}", inner.base_url, inner.zone_id);
            let response = inner
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", inner.api_token))
                .header("Content-Type", "application/json")
                .send()
                .context("Failed to send request to Cloudflare API")?;

            if !response.status().is_success() {
                return Ok(inner.zone_id.clone());
            }

            let api_response: ApiResponse<ZoneResponse> =
                response.json().context("Failed to parse API response")?;

            if let Some(zone) = api_response.result {
                Ok(zone.name)
            } else {
                Ok(inner.zone_id.clone())
            }
        })
        .await
    }

    pub async fn list_dns_records(&self) -> Result<Vec<DnsRecord>> {
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
                    .context("Failed to send request to Cloudflare API")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "API request failed with status {}: {}",
                        status,
                        body
                    ));
                }

                let api_response: ApiResponse<Vec<DnsRecordResponse>> =
                    response.json().context("Failed to parse API response")?;

                if !api_response.success {
                    let errors: Vec<String> = api_response
                        .errors
                        .iter()
                        .map(|e| format!("{} (code: {})", e.message, e.code))
                        .collect();
                    return Err(anyhow::anyhow!(
                        "Cloudflare API errors: {}",
                        errors.join(", ")
                    ));
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

    pub async fn create_dns_record(&self, record: &DnsRecord) -> Result<DnsRecord> {
        let inner = self.inner.clone();
        let record = record.clone();
        smol::unblock(move || {
            let url = format!(
                "{}/zones/{}/dns_records",
                inner.base_url, inner.zone_id
            );
            let response = inner
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", inner.api_token))
                .header("Content-Type", "application/json")
                .json(&record)
                .send()
                .context("Failed to send request to Cloudflare API")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "API request failed with status {}: {}",
                    status,
                    body
                ));
            }

            let api_response: ApiResponse<DnsRecordResponse> =
                response.json().context("Failed to parse API response")?;

            if !api_response.success {
                let errors: Vec<String> = api_response
                    .errors
                    .iter()
                    .map(|e| format!("{} (code: {})", e.message, e.code))
                    .collect();
                return Err(anyhow::anyhow!(
                    "Cloudflare API errors: {}",
                    errors.join(", ")
                ));
            }

            let result = api_response
                .result
                .ok_or_else(|| anyhow::anyhow!("No result returned from API"))?;

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

    pub async fn update_dns_record(&self, record: &DnsRecord) -> Result<DnsRecord> {
        let inner = self.inner.clone();
        let record = record.clone();
        smol::unblock(move || {
            let record_id = record
                .id
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Cannot update: record has no ID"))?;
            let url = format!(
                "{}/zones/{}/dns_records/{}",
                inner.base_url, inner.zone_id, record_id
            );
            let response = inner
                .client
                .put(&url)
                .header("Authorization", format!("Bearer {}", inner.api_token))
                .header("Content-Type", "application/json")
                .json(&record)
                .send()
                .context("Failed to send request to Cloudflare API")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "API request failed with status {}: {}",
                    status,
                    body
                ));
            }

            let api_response: ApiResponse<DnsRecordResponse> =
                response.json().context("Failed to parse API response")?;

            if !api_response.success {
                let errors: Vec<String> = api_response
                    .errors
                    .iter()
                    .map(|e| format!("{} (code: {})", e.message, e.code))
                    .collect();
                return Err(anyhow::anyhow!(
                    "Cloudflare API errors: {}",
                    errors.join(", ")
                ));
            }

            let result = api_response
                .result
                .ok_or_else(|| anyhow::anyhow!("No result returned from API"))?;

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

    pub async fn delete_dns_record(&self, record_id: &str) -> Result<()> {
        let inner = self.inner.clone();
        let record_id = record_id.to_string();
        smol::unblock(move || {
            let url = format!(
                "{}/zones/{}/dns_records/{}",
                inner.base_url, inner.zone_id, record_id
            );
            let response = inner
                .client
                .delete(&url)
                .header("Authorization", format!("Bearer {}", inner.api_token))
                .header("Content-Type", "application/json")
                .send()
                .context("Failed to send request to Cloudflare API")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "API request failed with status {}: {}",
                    status,
                    body
                ));
            }

            let api_response: ApiResponse<serde_json::Value> =
                response.json().context("Failed to parse API response")?;

            if !api_response.success {
                let errors: Vec<String> = api_response
                    .errors
                    .iter()
                    .map(|e| format!("{} (code: {})", e.message, e.code))
                    .collect();
                return Err(anyhow::anyhow!(
                    "Cloudflare API errors: {}",
                    errors.join(", ")
                ));
            }

            Ok(())
        })
        .await
    }
}
