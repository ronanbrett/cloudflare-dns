/// Formatting utilities for DNS records and UI display.
use std::collections::BTreeSet;
use std::fmt::Write;

use crate::api::DnsRecord;

/// Extract unique IP addresses from DNS records, filtering to only A/AAAA types.
pub fn extract_unique_ips(records: &[DnsRecord]) -> Vec<String> {
    let mut ips = BTreeSet::new();
    for r in records {
        if (r.record_type == "A" || r.record_type == "AAAA") && !ips.contains(&r.content) {
            ips.insert(r.content.clone());
        }
    }
    ips.into_iter().collect()
}

/// Strip the domain suffix from a DNS record name.
/// E.g., "pihole.robrett.com" with suffix ".robrett.com" -> "pihole"
pub fn strip_domain_suffix(name: &str, domain_suffix: &str) -> String {
    name.strip_suffix(domain_suffix).unwrap_or(name).to_string()
}

/// Format DNS records for display in a table format.
pub fn format_records(records: &[DnsRecord]) -> String {
    if records.is_empty() {
        return "No DNS records found".to_string();
    }
    let mut t = format!("{} DNS Records\n\n", records.len());
    for r in records {
        writeln!(
            t,
            "{:<6} | {:<30} | {:<20} | TTL: {:<6} | Proxy: {}",
            r.record_type,
            r.name,
            r.content,
            r.ttl.unwrap_or(0),
            if r.proxied.unwrap_or(false) {
                "Yes"
            } else {
                "No"
            }
        )
        .unwrap_or(());
    }
    t
}

/// Format a list of IPs for the selector UI with selection indicators.
pub fn format_selector(ips: &[String], selected_idx: usize) -> String {
    let mut s = String::new();
    for (i, ip) in ips.iter().enumerate() {
        s.push_str(&if i == selected_idx {
            format!("  ▸ {} ◂\n", ip)
        } else {
            format!("    {}\n", ip)
        });
    }
    let n = ips.len();
    s.push_str(&if n == selected_idx {
        "  ▸ ✎ Enter new IP address... ◂\n".to_string()
    } else {
        "    ✎ Enter new IP address...\n".to_string()
    });
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(id: &str, record_type: &str, name: &str, content: &str) -> DnsRecord {
        DnsRecord {
            id: Some(id.to_string()),
            record_type: record_type.to_string(),
            name: name.to_string(),
            content: content.to_string(),
            ttl: Some(300),
            proxied: Some(false),
            comment: None,
        }
    }

    #[test]
    fn test_extract_unique_ips_empty() {
        let records: Vec<DnsRecord> = vec![];
        let ips = extract_unique_ips(&records);
        assert!(ips.is_empty());
    }

    #[test]
    fn test_extract_unique_ips_a_records_only() {
        let records = vec![
            make_record("1", "A", "example.com", "192.168.1.1"),
            make_record("2", "A", "test.example.com", "192.168.1.2"),
        ];
        let ips = extract_unique_ips(&records);
        assert_eq!(ips, vec!["192.168.1.1", "192.168.1.2"]);
    }

    #[test]
    fn test_extract_unique_ips_aaaa_records_only() {
        let records = vec![
            make_record("1", "AAAA", "example.com", "2001:db8::1"),
            make_record("2", "AAAA", "test.example.com", "2001:db8::2"),
        ];
        let ips = extract_unique_ips(&records);
        assert_eq!(ips, vec!["2001:db8::1", "2001:db8::2"]);
    }

    #[test]
    fn test_extract_unique_ips_mixed_types() {
        let records = vec![
            make_record("1", "A", "example.com", "192.168.1.1"),
            make_record("2", "AAAA", "example.com", "2001:db8::1"),
            make_record("3", "CNAME", "www.example.com", "example.com"),
            make_record("4", "MX", "example.com", "mail.example.com"),
        ];
        let ips = extract_unique_ips(&records);
        assert_eq!(ips, vec!["192.168.1.1", "2001:db8::1"]);
    }

    #[test]
    fn test_extract_unique_ips_duplicates_removed() {
        let records = vec![
            make_record("1", "A", "example.com", "192.168.1.1"),
            make_record("2", "A", "test.example.com", "192.168.1.1"),
            make_record("3", "AAAA", "example.com", "2001:db8::1"),
            make_record("4", "AAAA", "test.example.com", "2001:db8::1"),
        ];
        let ips = extract_unique_ips(&records);
        assert_eq!(ips, vec!["192.168.1.1", "2001:db8::1"]);
    }

    #[test]
    fn test_extract_unique_ips_sorted() {
        let records = vec![
            make_record("1", "A", "example.com", "10.0.0.1"),
            make_record("2", "A", "test.example.com", "192.168.1.1"),
            make_record("3", "A", "dev.example.com", "172.16.0.1"),
        ];
        let ips = extract_unique_ips(&records);
        // BTreeSet should sort them
        assert_eq!(ips, vec!["10.0.0.1", "172.16.0.1", "192.168.1.1"]);
    }

    #[test]
    fn test_format_records_empty() {
        let records: Vec<DnsRecord> = vec![];
        let result = format_records(&records);
        assert_eq!(result, "No DNS records found");
    }

    #[test]
    fn test_format_records_single_record() {
        let records = vec![make_record("1", "A", "example.com", "192.168.1.1")];
        let result = format_records(&records);
        assert!(result.contains("1 DNS Records"));
        assert!(result.contains("A"));
        assert!(result.contains("example.com"));
        assert!(result.contains("192.168.1.1"));
        assert!(result.contains("TTL: 300"));
        assert!(result.contains("Proxy: No"));
    }

    #[test]
    fn test_format_records_multiple_records() {
        let records = vec![
            make_record("1", "A", "example.com", "192.168.1.1"),
            make_record("2", "AAAA", "example.com", "2001:db8::1"),
        ];
        let result = format_records(&records);
        assert!(result.contains("2 DNS Records"));
        assert!(result.contains("A"));
        assert!(result.contains("AAAA"));
    }

    #[test]
    fn test_format_records_proxied() {
        let records = vec![DnsRecord {
            id: Some("1".to_string()),
            record_type: "A".to_string(),
            name: "example.com".to_string(),
            content: "192.168.1.1".to_string(),
            ttl: Some(300),
            proxied: Some(true),
            comment: None,
        }];
        let result = format_records(&records);
        assert!(result.contains("Proxy: Yes"));
    }

    #[test]
    fn test_format_records_ttl_auto() {
        let records = vec![DnsRecord {
            id: Some("1".to_string()),
            record_type: "A".to_string(),
            name: "example.com".to_string(),
            content: "192.168.1.1".to_string(),
            ttl: Some(1),
            proxied: Some(false),
            comment: None,
        }];
        let result = format_records(&records);
        assert!(result.contains("TTL: 1"));
    }

    #[test]
    fn test_format_selector_single_ip() {
        let ips = vec!["192.168.1.1".to_string()];
        let result = format_selector(&ips, 0);
        assert!(result.contains("▸ 192.168.1.1 ◂"));
        assert!(result.contains("Enter new IP address"));
    }

    #[test]
    fn test_format_selector_multiple_ips() {
        let ips = vec![
            "192.168.1.1".to_string(),
            "192.168.1.2".to_string(),
            "192.168.1.3".to_string(),
        ];
        let result = format_selector(&ips, 0);
        assert!(result.contains("▸ 192.168.1.1 ◂"));
        // Other IPs should not have highlight
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines[1].starts_with("    192.168.1.2"));
        assert!(lines[2].starts_with("    192.168.1.3"));
    }

    #[test]
    fn test_format_selector_selected_last() {
        let ips = vec!["192.168.1.1".to_string(), "192.168.1.2".to_string()];
        let result = format_selector(&ips, 1);
        assert!(result.contains("▸ 192.168.1.2 ◂"));
        // "Enter new IP" should not be highlighted when selected_idx < ips.len()
        assert!(result.contains("    ✎ Enter new IP address"));
    }

    #[test]
    fn test_format_selector_enter_new_ip_selected() {
        let ips = vec!["192.168.1.1".to_string()];
        let result = format_selector(&ips, 1);
        assert!(result.contains("▸ ✎ Enter new IP address... ◂"));
    }

    #[test]
    fn test_strip_domain_suffix_basic() {
        assert_eq!(
            strip_domain_suffix("test.example.com", "example.com"),
            "test."
        );
    }

    #[test]
    fn test_strip_domain_suffix_no_match() {
        assert_eq!(
            strip_domain_suffix("other-domain.org", "example.com"),
            "other-domain.org"
        );
    }

    #[test]
    fn test_strip_domain_suffix_exact_match() {
        assert_eq!(strip_domain_suffix("example.com", "example.com"), "");
    }

    #[test]
    fn test_strip_domain_suffix_empty_suffix() {
        assert_eq!(strip_domain_suffix("example.com", ""), "example.com");
    }

    #[test]
    fn test_strip_domain_suffix_subdomain() {
        assert_eq!(
            strip_domain_suffix("www.api.example.com", "example.com"),
            "www.api."
        );
    }
}
