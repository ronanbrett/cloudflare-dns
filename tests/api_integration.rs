use cloudflaredns::cloudflare::CloudflareClient;
use mockito::Server;

fn create_test_client(mock_server: &Server) -> CloudflareClient {
    CloudflareClient::with_base_url(
        "test_api_token".to_string(),
        "test_zone_123".to_string(),
        mock_server.url(),
    )
}

// ─── get_zone_name ───────────────────────────────────────────────────────────

#[test]
fn test_get_zone_name_success() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/zones/test_zone_123")
            .match_header("Authorization", "Bearer test_api_token")
            .with_status(200)
            .with_body(
                r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": {
                    "id": "test_zone_123",
                    "name": "example.com"
                }
            }"#,
            )
            .create();

        let client = create_test_client(&server);
        let zone_name = client.get_zone_name().await.unwrap();
        assert_eq!(zone_name, "example.com");
    });
}

#[test]
fn test_get_zone_name_falls_back_to_zone_id() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/zones/test_zone_123")
            .with_status(404)
            .with_body(
                r#"{"success": false, "errors": [{"code": 1001, "message": "Not found"}]}"#,
            )
            .create();

        let client = create_test_client(&server);
        let zone_name = client.get_zone_name().await.unwrap();
        assert_eq!(zone_name, "test_zone_123");
    });
}

// ─── list_dns_records ────────────────────────────────────────────────────────

#[test]
fn test_list_dns_records_empty() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/zones/test_zone_123/dns_records?page=1&per_page=100",
            )
            .with_status(200)
            .with_body(
                r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": []
            }"#,
            )
            .create();

        let client = create_test_client(&server);
        let records: Vec<_> = client.list_dns_records().await.unwrap();
        assert!(records.is_empty());
    });
}

#[test]
fn test_list_dns_records_single_page() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/zones/test_zone_123/dns_records?page=1&per_page=100",
            )
            .with_status(200)
            .with_body(
                r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": [
                    {
                        "id": "rec_001",
                        "type": "A",
                        "name": "example.com",
                        "content": "192.168.1.1",
                        "ttl": 300,
                        "proxied": false
                    },
                    {
                        "id": "rec_002",
                        "type": "AAAA",
                        "name": "example.com",
                        "content": "2001:db8::1",
                        "ttl": 300,
                        "proxied": true,
                        "comment": "IPv6 record"
                    }
                ]
            }"#,
            )
            .create();

        let client = create_test_client(&server);
        let records: Vec<_> = client.list_dns_records().await.unwrap();
        assert_eq!(records.len(), 2);

        assert_eq!(records[0].record_type, "A");
        assert_eq!(records[0].content, "192.168.1.1");
        assert_eq!(records[0].proxied, Some(false));

        assert_eq!(records[1].record_type, "AAAA");
        assert_eq!(records[1].content, "2001:db8::1");
        assert_eq!(records[1].proxied, Some(true));
        assert_eq!(records[1].comment, Some("IPv6 record".to_string()));
    });
}

#[test]
fn test_list_dns_records_api_error() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/zones/test_zone_123/dns_records?page=1&per_page=100",
            )
            .with_status(400)
            .with_body("Bad Request")
            .create();

        let client = create_test_client(&server);
        let result: Result<Vec<_>, _> = client.list_dns_records().await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("400"));
    });
}

#[test]
fn test_list_dns_records_api_returns_error_response() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock(
                "GET",
                "/zones/test_zone_123/dns_records?page=1&per_page=100",
            )
            .with_status(200)
            .with_body(
                r#"{
                "success": false,
                "errors": [{"code": 9001, "message": "Invalid API token"}],
                "messages": [],
                "result": []
            }"#,
            )
            .create();

        let client = create_test_client(&server);
        let result: Result<Vec<_>, _> = client.list_dns_records().await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid API token"));
    });
}

// ─── create_dns_record ───────────────────────────────────────────────────────

#[test]
fn test_create_dns_record_success() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("POST", "/zones/test_zone_123/dns_records")
            .match_header("Authorization", "Bearer test_api_token")
            .match_header("Content-Type", "application/json")
            .with_status(200)
            .with_body(
                r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": {
                    "id": "new_rec_001",
                    "type": "A",
                    "name": "new.example.com",
                    "content": "10.0.0.1",
                    "ttl": 300,
                    "proxied": false
                }
            }"#,
            )
            .create();

        let client = create_test_client(&server);
        let record = cloudflaredns::cloudflare::DnsRecord {
            id: None,
            record_type: "A".to_string(),
            name: "new.example.com".to_string(),
            content: "10.0.0.1".to_string(),
            ttl: Some(300),
            proxied: Some(false),
            comment: None,
        };

        let result = client.create_dns_record(&record).await.unwrap();
        assert_eq!(result.id, Some("new_rec_001".to_string()));
        assert_eq!(result.record_type, "A");
        assert_eq!(result.content, "10.0.0.1");
    });
}

// ─── update_dns_record ───────────────────────────────────────────────────────

#[test]
fn test_update_dns_record_success() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("PUT", "/zones/test_zone_123/dns_records/existing_rec_001")
            .with_status(200)
            .with_body(
                r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": {
                    "id": "existing_rec_001",
                    "type": "A",
                    "name": "updated.example.com",
                    "content": "10.0.0.2",
                    "ttl": 600,
                    "proxied": true
                }
            }"#,
            )
            .create();

        let client = create_test_client(&server);
        let record = cloudflaredns::cloudflare::DnsRecord {
            id: Some("existing_rec_001".to_string()),
            record_type: "A".to_string(),
            name: "updated.example.com".to_string(),
            content: "10.0.0.2".to_string(),
            ttl: Some(600),
            proxied: Some(true),
            comment: None,
        };

        let result = client.update_dns_record(&record).await.unwrap();
        assert_eq!(result.id, Some("existing_rec_001".to_string()));
        assert_eq!(result.content, "10.0.0.2");
        assert_eq!(result.ttl, Some(600));
        assert_eq!(result.proxied, Some(true));
    });
}

#[test]
fn test_update_dns_record_fails_without_id() {
    smol::block_on(async {
        let server = Server::new_async().await;
        let client = create_test_client(&server);
        let record = cloudflaredns::cloudflare::DnsRecord {
            id: None,
            record_type: "A".to_string(),
            name: "test.example.com".to_string(),
            content: "10.0.0.1".to_string(),
            ttl: Some(300),
            proxied: Some(false),
            comment: None,
        };

        let result = client.update_dns_record(&record).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("no ID"));
    });
}

// ─── delete_dns_record ───────────────────────────────────────────────────────

#[test]
fn test_delete_dns_record_success() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("DELETE", "/zones/test_zone_123/dns_records/rec_to_delete")
            .with_status(200)
            .with_body(
                r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": null
            }"#,
            )
            .create();

        let client = create_test_client(&server);
        let result: Result<(), anyhow::Error> = client.delete_dns_record("rec_to_delete").await;
        assert!(result.is_ok());
    });
}

// ─── Authorization header ────────────────────────────────────────────────────

#[test]
fn test_client_sends_correct_auth_header() {
    smol::block_on(async {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/zones/test_zone_123")
            .match_header("Authorization", "Bearer my_secret_token")
            .with_status(200)
            .with_body(
                r#"{
                "success": true,
                "errors": [],
                "messages": [],
                "result": {
                    "id": "test_zone_123",
                    "name": "example.com"
                }
            }"#,
            )
            .create();

        let client = CloudflareClient::with_base_url(
            "my_secret_token".to_string(),
            "test_zone_123".to_string(),
            server.url(),
        );
        let zone_name = client.get_zone_name().await.unwrap();
        assert_eq!(zone_name, "example.com");
    });
}
