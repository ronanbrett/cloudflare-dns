/// Integration tests for the Cloudflare API client using mockito.
///
/// These tests verify that the client correctly handles API responses,
/// errors, and pagination without making real network requests.
use cloudflaredns::api::CloudflareClient;
use cloudflaredns::api::DnsRecord;

#[test]
fn test_list_dns_records_success() {
    let mut mock_server = mockito::Server::new();

    let _mock = mock_server
        .mock("GET", "/zones/zone123/dns_records")
        .match_query("page=1&per_page=100")
        .match_header("Authorization", "Bearer test_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": [
                    {
                        "id": "rec1",
                        "type": "A",
                        "name": "example.com",
                        "content": "192.168.1.1",
                        "ttl": 300,
                        "proxied": false
                    }
                ]
            }"#,
        )
        .create();

    let client = CloudflareClient::with_base_url(
        "test_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    let records = smol::block_on(client.list_dns_records()).unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, Some("rec1".to_string()));
    assert_eq!(records[0].record_type, "A");
    assert_eq!(records[0].name, "example.com");
    assert_eq!(records[0].content, "192.168.1.1");
}

#[test]
fn test_list_dns_records_empty() {
    let mut mock_server = mockito::Server::new();

    let _mock = mock_server
        .mock("GET", "/zones/zone123/dns_records")
        .match_query("page=1&per_page=100")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": []
            }"#,
        )
        .create();

    let client = CloudflareClient::with_base_url(
        "test_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    let records = smol::block_on(client.list_dns_records()).unwrap();
    assert!(records.is_empty());
}

#[test]
fn test_list_dns_records_api_error() {
    let mut mock_server = mockito::Server::new();

    let _mock = mock_server
        .mock("GET", "/zones/zone123/dns_records")
        .match_query(mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "success": false,
                "errors": [{"code": 1000, "message": "Invalid API token"}],
                "messages": [],
                "result": null
            }"#,
        )
        .create();

    let client = CloudflareClient::with_base_url(
        "invalid_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    let result = smol::block_on(client.list_dns_records());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid API token"));
}

#[test]
fn test_list_dns_records_http_error() {
    let mut mock_server = mockito::Server::new();

    let _mock = mock_server
        .mock("GET", "/zones/zone123/dns_records")
        .match_query(mockito::Matcher::Any)
        .with_status(501)
        .with_body("Server error")
        .create();

    let client = CloudflareClient::with_base_url(
        "test_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    let result = smol::block_on(client.list_dns_records());
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("501"));
}

#[test]
fn test_create_dns_record_success() {
    let mut mock_server = mockito::Server::new();

    let _mock = mock_server
        .mock("POST", "/zones/zone123/dns_records")
        .match_header("Authorization", "Bearer test_token")
        .match_header("content-type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": {
                    "id": "new_rec",
                    "type": "A",
                    "name": "new.example.com",
                    "content": "10.0.0.1",
                    "ttl": 300,
                    "proxied": true
                }
            }"#,
        )
        .create();

    let client = CloudflareClient::with_base_url(
        "test_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    let new_record = DnsRecord {
        id: None,
        record_type: "A".to_string(),
        name: "new.example.com".to_string(),
        content: "10.0.0.1".to_string(),
        ttl: Some(300),
        proxied: Some(true),
        comment: None,
    };

    let result = smol::block_on(client.create_dns_record(&new_record)).unwrap();
    assert_eq!(result.id, Some("new_rec".to_string()));
    assert_eq!(result.record_type, "A");
    assert_eq!(result.name, "new.example.com");
    assert_eq!(result.content, "10.0.0.1");
    assert_eq!(result.ttl, Some(300));
    assert_eq!(result.proxied, Some(true));
}

#[test]
fn test_delete_dns_record_success() {
    let mut mock_server = mockito::Server::new();

    let _mock = mock_server
        .mock("DELETE", "/zones/zone123/dns_records/rec123")
        .match_header("Authorization", "Bearer test_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": null
            }"#,
        )
        .create();

    let client = CloudflareClient::with_base_url(
        "test_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    let result = smol::block_on(client.delete_dns_record("rec123"));
    assert!(result.is_ok());
}

#[test]
fn test_update_dns_record_success() {
    let mut mock_server = mockito::Server::new();

    let _mock = mock_server
        .mock("PUT", "/zones/zone123/dns_records/rec123")
        .match_header("Authorization", "Bearer test_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": {
                    "id": "rec123",
                    "type": "A",
                    "name": "updated.example.com",
                    "content": "10.0.0.2",
                    "ttl": 600,
                    "proxied": false
                }
            }"#,
        )
        .create();

    let client = CloudflareClient::with_base_url(
        "test_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    let record = DnsRecord {
        id: Some("rec123".to_string()),
        record_type: "A".to_string(),
        name: "updated.example.com".to_string(),
        content: "10.0.0.2".to_string(),
        ttl: Some(600),
        proxied: Some(false),
        comment: None,
    };

    let result = smol::block_on(client.update_dns_record(&record)).unwrap();
    assert_eq!(result.id, Some("rec123".to_string()));
    assert_eq!(result.name, "updated.example.com");
    assert_eq!(result.content, "10.0.0.2");
    assert_eq!(result.ttl, Some(600));
}

#[test]
fn test_get_zone_name_success() {
    let mut mock_server = mockito::Server::new();

    let _mock = mock_server
        .mock("GET", "/zones/zone123")
        .match_header("Authorization", "Bearer test_token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": {
                    "id": "zone123",
                    "name": "example.com"
                }
            }"#,
        )
        .create();

    let client = CloudflareClient::with_base_url(
        "test_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    let zone_name = smol::block_on(client.get_zone_name()).unwrap();
    assert_eq!(zone_name, "example.com");
}

#[test]
fn test_get_zone_name_fallback_to_id() {
    let mut mock_server = mockito::Server::new();

    let _mock = mock_server
        .mock("GET", "/zones/zone123")
        .with_status(404)
        .with_body("Not found")
        .create();

    let client = CloudflareClient::with_base_url(
        "test_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    // Should fallback to zone ID on error
    let zone_name = smol::block_on(client.get_zone_name()).unwrap();
    assert_eq!(zone_name, "zone123");
}

#[test]
fn test_update_dns_record_missing_id() {
    let mut mock_server = mockito::Server::new();

    let client = CloudflareClient::with_base_url(
        "test_token".to_string(),
        "zone123".to_string(),
        mock_server.url(),
    );

    let record = DnsRecord {
        id: None, // No ID provided
        record_type: "A".to_string(),
        name: "example.com".to_string(),
        content: "10.0.0.1".to_string(),
        ttl: Some(300),
        proxied: Some(false),
        comment: None,
    };

    let result = smol::block_on(client.update_dns_record(&record));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("no ID provided"));
}
