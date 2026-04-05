use std::collections::BTreeSet;
use std::fmt::Write;

use crate::cloudflare::DnsRecord;

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
