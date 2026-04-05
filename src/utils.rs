use crate::cloudflare::DnsRecord;

pub fn format_records(records: &[DnsRecord]) -> String {
    if records.is_empty() {
        return "No DNS records found".to_string();
    }
    let mut t = format!("{} DNS Records\n\n", records.len());
    for r in records {
        t.push_str(&format!(
            "{:<6} | {:<30} | {:<20} | TTL: {:<6} | Proxy: {}\n",
            r.record_type,
            r.name,
            r.content,
            r.ttl.unwrap_or(0),
            if r.proxied.unwrap_or(false) {
                "Yes"
            } else {
                "No"
            }
        ));
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
