/// Data models for Cloudflare DNS records and API responses.
///
/// This module contains all the types used for serializing and deserializing
/// Cloudflare API requests and responses.
use serde::{Deserialize, Serialize};

/// Represents a DNS record in Cloudflare's system.
///
/// This is the primary data structure used throughout the application.
/// The `id` field is `Option<String>` because it's not present when creating new records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    /// The unique identifier for the record (not present when creating new records)
    pub id: Option<String>,

    /// The DNS record type (A, AAAA, CNAME, MX, TXT, etc.)
    #[serde(rename = "type")]
    pub record_type: String,

    /// The DNS record name (e.g., "example.com" or "www.example.com")
    pub name: String,

    /// The record content (IP address, hostname, etc.)
    pub content: String,

    /// Time-to-live in seconds (use `1` for automatic)
    pub ttl: Option<i64>,

    /// Whether the record is proxied through Cloudflare (orange cloud)
    pub proxied: Option<bool>,

    /// Optional comment for the record
    pub comment: Option<String>,
}

/// Generic API response wrapper from Cloudflare.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,

    /// Error details if the request failed
    pub errors: Vec<ApiError>,

    /// Informational messages
    pub messages: Vec<String>,

    /// The response data
    pub result: Option<T>,
}

/// Individual error from Cloudflare API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: i64,

    /// Error message
    pub message: String,
}

/// DNS record as returned by the Cloudflare API (always has an ID).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecordResponse {
    pub id: String,

    #[serde(rename = "type")]
    pub record_type: String,

    pub name: String,

    pub content: String,

    pub ttl: i64,

    pub proxied: bool,

    pub comment: Option<String>,
}

/// Zone information from Cloudflare.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneResponse {
    pub id: String,

    pub name: String,
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
