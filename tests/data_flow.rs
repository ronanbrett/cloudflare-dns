use cloudflaredns::api::DnsRecord;
use cloudflaredns::ui::state::AppView;
use cloudflaredns::ui::status::{StatusMessage, StatusType, generate_contextual_status};
use cloudflaredns::utils::{
    extract_unique_ips, format_records, format_selector, strip_domain_suffix,
};

// ─── Realistic DNS record data flow ──────────────────────────────────────────

fn make_test_records() -> Vec<DnsRecord> {
    vec![
        DnsRecord {
            id: Some("rec_001".to_string()),
            record_type: "A".to_string(),
            name: "example.com".to_string(),
            content: "192.168.1.1".to_string(),
            ttl: Some(300),
            proxied: Some(true),
            comment: Some("Main website".to_string()),
        },
        DnsRecord {
            id: Some("rec_002".to_string()),
            record_type: "AAAA".to_string(),
            name: "example.com".to_string(),
            content: "2001:db8::1".to_string(),
            ttl: Some(300),
            proxied: Some(true),
            comment: Some("Main website IPv6".to_string()),
        },
        DnsRecord {
            id: Some("rec_003".to_string()),
            record_type: "CNAME".to_string(),
            name: "www.example.com".to_string(),
            content: "example.com".to_string(),
            ttl: Some(3600),
            proxied: Some(false),
            comment: None,
        },
        DnsRecord {
            id: Some("rec_004".to_string()),
            record_type: "MX".to_string(),
            name: "example.com".to_string(),
            content: "mail.example.com".to_string(),
            ttl: Some(3600),
            proxied: Some(false),
            comment: None,
        },
        DnsRecord {
            id: Some("rec_005".to_string()),
            record_type: "TXT".to_string(),
            name: "example.com".to_string(),
            content: "v=spf1 include:_spf.example.com ~all".to_string(),
            ttl: Some(3600),
            proxied: Some(false),
            comment: Some("SPF record".to_string()),
        },
    ]
}

#[test]
fn test_extract_ips_from_realistic_records() {
    let records = make_test_records();
    let ips = extract_unique_ips(&records);

    // Only A and AAAA records should be included
    assert_eq!(ips.len(), 2);
    assert!(ips.contains(&"192.168.1.1".to_string()));
    assert!(ips.contains(&"2001:db8::1".to_string()));
    // CNAME, MX, and TXT content should be excluded
}

#[test]
fn test_format_records_table_from_realistic_data() {
    let records = make_test_records();
    let output = format_records(&records);

    // Should include the count header
    assert!(output.contains("5 DNS Records"));

    // Should include record types
    assert!(output.contains("A"));
    assert!(output.contains("AAAA"));
    assert!(output.contains("CNAME"));
    assert!(output.contains("MX"));
    assert!(output.contains("TXT"));

    // Should include IP addresses
    assert!(output.contains("192.168.1.1"));
    assert!(output.contains("2001:db8::1"));

    // Should include proxy status
    assert!(output.contains("Proxy: Yes"));
    assert!(output.contains("Proxy: No"));
}

#[test]
fn test_selector_from_records_flow() {
    let records = make_test_records();
    let ips = extract_unique_ips(&records);

    // Should have 2 IPs
    assert_eq!(ips.len(), 2);

    // Format the selector with first IP selected
    let selector = format_selector(&ips, 0);
    assert!(selector.contains("▸ 192.168.1.1 ◂"));
    assert!(selector.contains("2001:db8::1"));
    assert!(selector.contains("Enter new IP address"));
}

#[test]
fn test_selector_with_no_a_aaaa_records() {
    let records = vec![
        DnsRecord {
            id: Some("rec_001".to_string()),
            record_type: "CNAME".to_string(),
            name: "www.example.com".to_string(),
            content: "example.com".to_string(),
            ttl: Some(3600),
            proxied: Some(false),
            comment: None,
        },
        DnsRecord {
            id: Some("rec_002".to_string()),
            record_type: "MX".to_string(),
            name: "example.com".to_string(),
            content: "mail.example.com".to_string(),
            ttl: Some(3600),
            proxied: Some(false),
            comment: None,
        },
    ];

    let ips = extract_unique_ips(&records);
    assert!(ips.is_empty());

    // Selector should still show the "Enter new IP" option
    let selector = format_selector(&ips, 0);
    assert!(selector.contains("Enter new IP address"));
    assert!(selector.contains("▸"));
}

// ─── Strip domain suffix with realistic names ────────────────────────────────

#[test]
fn test_strip_suffix_realistic_subdomains() {
    let domain = ".example.com";

    assert_eq!(strip_domain_suffix("www", domain), "www");
    assert_eq!(strip_domain_suffix("www.example.com", domain), "www");
    assert_eq!(strip_domain_suffix("api.example.com", domain), "api");
    assert_eq!(
        strip_domain_suffix("staging.api.example.com", domain),
        "staging.api"
    );
    // "example.com" doesn't end with ".example.com" (missing leading dot)
    assert_eq!(strip_domain_suffix("example.com", domain), "example.com");
}

#[test]
fn test_strip_suffix_without_leading_dot() {
    let domain = "example.com";

    assert_eq!(strip_domain_suffix("www.example.com", domain), "www.");
    assert_eq!(strip_domain_suffix("example.com", domain), "");
}

// ─── Status messages with realistic data flow ────────────────────────────────

#[test]
fn test_status_for_loaded_records() {
    let records = make_test_records();
    let status = StatusMessage::OperationResult(format!("Loaded {} DNS records", records.len()));
    let rendered = status.render();

    assert_eq!(rendered, "Loaded 5 DNS records");
}

#[test]
fn test_status_for_created_record() {
    let status = StatusMessage::OperationResult("Created A for www".to_string());
    let rendered = status.render();

    assert_eq!(rendered, "Created A for www");
    assert_eq!(status.status_type(), StatusType::Transient);
}

#[test]
fn test_status_for_record_list_with_realistic_data() {
    let records = make_test_records();
    let status = generate_contextual_status(
        &AppView::List,
        0,
        "A",
        "false",
        false,
        records.len(),
        2,
        Some("www.example.com"),
    );

    let rendered = status.render();
    assert!(rendered.contains("3 of 5"));
    assert!(rendered.contains("www.example.com"));
    assert!(rendered.contains("E: edit"));
    assert!(rendered.contains("D: delete"));
}

#[test]
fn test_status_for_edit_form_all_fields() {
    let field_statuses = [
        generate_contextual_status(&AppView::Edit, 0, "A", "false", true, 5, 0, None),
        generate_contextual_status(&AppView::Edit, 1, "A", "false", true, 5, 0, None),
        generate_contextual_status(&AppView::Edit, 2, "A", "false", true, 5, 0, None),
        generate_contextual_status(&AppView::Edit, 3, "A", "false", true, 5, 0, None),
        generate_contextual_status(&AppView::Edit, 4, "A", "true", true, 5, 0, None),
        generate_contextual_status(&AppView::Edit, 5, "A", "false", true, 5, 0, None),
    ];

    let rendered: Vec<String> = field_statuses
        .iter()
        .map(|s: &StatusMessage| s.render())
        .collect();

    assert!(rendered[0].contains("Type: A"));
    assert!(rendered[0].contains("Field 1/6"));
    assert!(rendered[1].contains("Field 2/6"));
    assert!(rendered[2].contains("IP Address"));
    assert!(rendered[2].contains("Field 3/6"));
    assert!(rendered[3].contains("TTL"));
    assert!(rendered[3].contains("Field 4/6"));
    assert!(rendered[4].contains("Orange cloud ON"));
    assert!(rendered[4].contains("Field 5/6"));
    assert!(rendered[5].contains("Save"));
    assert!(rendered[5].contains("Field 6/6"));
}

// ─── End-to-end: API response to display ─────────────────────────────────────

#[test]
fn test_api_response_to_formatted_table() {
    // Simulate JSON from Cloudflare API
    let api_json = r#"[
        {
            "id": "cf_rec_1",
            "type": "A",
            "name": "api.example.com",
            "content": "10.0.0.1",
            "ttl": 1,
            "proxied": true,
            "comment": "API server"
        },
        {
            "id": "cf_rec_2",
            "type": "AAAA",
            "name": "api.example.com",
            "content": "2001:db8:dead::1",
            "ttl": 1,
            "proxied": true
        }
    ]"#;

    // Parse JSON into DnsRecord structs
    let records: Vec<DnsRecord> = serde_json::from_str(api_json).unwrap();

    // Extract IPs
    let ips = extract_unique_ips(&records);
    assert_eq!(ips.len(), 2);
    assert!(ips.contains(&"10.0.0.1".to_string()));
    assert!(ips.contains(&"2001:db8:dead::1".to_string()));

    // Format for display
    let table = format_records(&records);
    assert!(table.contains("2 DNS Records"));
    assert!(table.contains("api.example.com"));
    assert!(table.contains("Proxy: Yes"));
}

#[test]
fn test_empty_list_to_create_status_flow() {
    let empty_records: Vec<DnsRecord> = vec![];
    let ips = extract_unique_ips(&empty_records);
    assert!(ips.is_empty());

    let table = format_records(&empty_records);
    assert_eq!(table, "No DNS records found");

    let status = generate_contextual_status(&AppView::List, 0, "A", "false", false, 0, 0, None);
    let rendered = status.render();
    assert!(rendered.contains("C: create your first"));
}

#[test]
fn test_record_selection_status_flow() {
    let records = make_test_records();

    // Test selecting each record
    for (i, record) in records.iter().enumerate() {
        let status = generate_contextual_status(
            &AppView::List,
            0,
            "A",
            "false",
            false,
            records.len(),
            i,
            Some(&record.name),
        );

        let rendered = status.render();
        assert!(rendered.contains(&format!("{} of 5", i + 1)));
        assert!(rendered.contains(&record.name));
    }
}

// ─── Edge cases in data flow ─────────────────────────────────────────────────

#[test]
fn test_special_characters_in_record_names() {
    let records = vec![
        DnsRecord {
            id: Some("srv_001".to_string()),
            record_type: "SRV".to_string(),
            name: "_sip._tcp.example.com".to_string(),
            content: "10 60 5060 sip.example.com".to_string(),
            ttl: Some(3600),
            proxied: Some(false),
            comment: None,
        },
        DnsRecord {
            id: Some("caa_001".to_string()),
            record_type: "CAA".to_string(),
            name: "example.com".to_string(),
            content: "0 issue \"letsencrypt.org\"".to_string(),
            ttl: Some(3600),
            proxied: Some(false),
            comment: None,
        },
    ];

    let table = format_records(&records);
    assert!(table.contains("_sip._tcp.example.com"));
    assert!(table.contains("letsencrypt.org"));

    // SRV records should not contribute IPs
    let ips = extract_unique_ips(&records);
    assert!(ips.is_empty());
}

#[test]
fn test_duplicate_ips_across_different_record_types() {
    // Same IP in both A and AAAA (shouldn't happen in reality, but test dedup)
    let records = vec![
        DnsRecord {
            id: Some("a1".to_string()),
            record_type: "A".to_string(),
            name: "site1.example.com".to_string(),
            content: "1.2.3.4".to_string(),
            ttl: Some(300),
            proxied: Some(false),
            comment: None,
        },
        DnsRecord {
            id: Some("a2".to_string()),
            record_type: "A".to_string(),
            name: "site2.example.com".to_string(),
            content: "1.2.3.4".to_string(),
            ttl: Some(300),
            proxied: Some(false),
            comment: None,
        },
    ];

    let ips = extract_unique_ips(&records);
    assert_eq!(ips.len(), 1);
    assert_eq!(ips[0], "1.2.3.4");
}
