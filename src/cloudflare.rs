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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_record_serialize_minimal() {
        let record = DnsRecord {
            id: None,
            record_type: "A".to_string(),
            name: "example.com".to_string(),
            content: "192.168.1.1".to_string(),
            ttl: None,
            proxied: None,
            comment: None,
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"type\":\"A\""));
        assert!(json.contains("\"name\":\"example.com\""));
        assert!(json.contains("\"content\":\"192.168.1.1\""));
    }

    #[test]
    fn test_dns_record_serialize_full() {
        let record = DnsRecord {
            id: Some("abc123".to_string()),
            record_type: "AAAA".to_string(),
            name: "test.example.com".to_string(),
            content: "2001:db8::1".to_string(),
            ttl: Some(300),
            proxied: Some(true),
            comment: Some("Test record".to_string()),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"id\":\"abc123\""));
        assert!(json.contains("\"type\":\"AAAA\""));
        assert!(json.contains("\"proxied\":true"));
        assert!(json.contains("\"comment\":\"Test record\""));
    }

    #[test]
    fn test_dns_record_deserialize() {
        let json = r#"{
            "id": "def456",
            "type": "CNAME",
            "name": "www.example.com",
            "content": "example.com",
            "ttl": 3600,
            "proxied": false
        }"#;
        let record: DnsRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.id, Some("def456".to_string()));
        assert_eq!(record.record_type, "CNAME");
        assert_eq!(record.name, "www.example.com");
        assert_eq!(record.content, "example.com");
        assert_eq!(record.ttl, Some(3600));
        assert_eq!(record.proxied, Some(false));
    }

    #[test]
    fn test_dns_record_deserialize_with_comment() {
        let json = r#"{
            "id": "ghi789",
            "type": "A",
            "name": "api.example.com",
            "content": "10.0.0.1",
            "ttl": 1,
            "proxied": true,
            "comment": "API server"
        }"#;
        let record: DnsRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.comment, Some("API server".to_string()));
    }

    #[test]
    fn test_dns_record_deserialize_without_comment() {
        let json = r#"{
            "id": "jkl012",
            "type": "MX",
            "name": "example.com",
            "content": "mail.example.com",
            "ttl": 3600,
            "proxied": false
        }"#;
        let record: DnsRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.comment, None);
    }

    #[test]
    fn test_dns_record_type_renamed_correctly() {
        let json = r#"{"type": "TXT", "name": "example.com", "content": "v=spf1"}"#;
        let record: DnsRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.record_type, "TXT");
    }

    #[test]
    fn test_api_response_structure() {
        let json = r#"{
            "success": true,
            "errors": [],
            "messages": [],
            "result": {
                "id": "zone123",
                "name": "example.com"
            }
        }"#;
        let response: ApiResponse<ZoneResponse> = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert!(response.errors.is_empty());
        assert!(response.messages.is_empty());
        assert!(response.result.is_some());
        let zone = response.result.unwrap();
        assert_eq!(zone.id, "zone123");
        assert_eq!(zone.name, "example.com");
    }

    #[test]
    fn test_api_response_with_errors() {
        let json = r#"{
            "success": false,
            "errors": [{"code": 1000, "message": "Invalid API token"}],
            "messages": [],
            "result": null
        }"#;
        let response: ApiResponse<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert!(!response.success);
        assert_eq!(response.errors.len(), 1);
        assert_eq!(response.errors[0].code, 1000);
        assert_eq!(response.errors[0].message, "Invalid API token");
    }

    #[test]
    fn test_dns_record_response_structure() {
        let json = r#"{
            "id": "rec123",
            "type": "A",
            "name": "example.com",
            "content": "192.168.1.1",
            "ttl": 300,
            "proxied": false,
            "comment": "Test"
        }"#;
        let record: DnsRecordResponse = serde_json::from_str(json).unwrap();
        assert_eq!(record.id, "rec123");
        assert_eq!(record.record_type, "A");
        assert_eq!(record.content, "192.168.1.1");
        assert_eq!(record.ttl, 300);
        assert!(!record.proxied);
        assert_eq!(record.comment, Some("Test".to_string()));
    }

    #[test]
    fn test_zone_response_structure() {
        let json = r#"{
            "id": "zone456",
            "name": "mydomain.org"
        }"#;
        let zone: ZoneResponse = serde_json::from_str(json).unwrap();
        assert_eq!(zone.id, "zone456");
        assert_eq!(zone.name, "mydomain.org");
    }

    #[test]
    fn test_dns_record_roundtrip() {
        let original = DnsRecord {
            id: Some("rt001".to_string()),
            record_type: "SRV".to_string(),
            name: "_sip._tcp.example.com".to_string(),
            content: "10 60 5060 sip.example.com".to_string(),
            ttl: Some(3600),
            proxied: Some(false),
            comment: Some("SIP service".to_string()),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: DnsRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, original.id);
        assert_eq!(deserialized.record_type, original.record_type);
        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.content, original.content);
        assert_eq!(deserialized.ttl, original.ttl);
        assert_eq!(deserialized.proxied, original.proxied);
        assert_eq!(deserialized.comment, original.comment);
    }

    #[test]
    fn test_dns_record_clone() {
        let record = DnsRecord {
            id: Some("clone001".to_string()),
            record_type: "A".to_string(),
            name: "test.example.com".to_string(),
            content: "10.0.0.1".to_string(),
            ttl: Some(60),
            proxied: Some(true),
            comment: None,
        };
        let cloned = record.clone();
        assert_eq!(cloned.id, record.id);
        assert_eq!(cloned.record_type, record.record_type);
        assert_eq!(cloned.content, record.content);
    }

    #[test]
    fn test_dns_record_debug_format() {
        let record = DnsRecord {
            id: Some("debug001".to_string()),
            record_type: "NS".to_string(),
            name: "example.com".to_string(),
            content: "ns1.example.com".to_string(),
            ttl: Some(86400),
            proxied: Some(false),
            comment: None,
        };
        let debug_str = format!("{:?}", record);
        assert!(debug_str.contains("debug001"));
        assert!(debug_str.contains("NS"));
        assert!(debug_str.contains("ns1.example.com"));
    }
}
