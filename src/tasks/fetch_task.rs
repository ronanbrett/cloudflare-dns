/// Fetch and refresh tasks for loading DNS records.
use iocraft::prelude::*;

use crate::api::CloudflareClient;
use crate::ui::state::AppState;
use crate::utils::{extract_unique_ips, format_records};

/// Fetch all initial data: zone name and DNS records.
///
/// Uses the DNS cache if valid to avoid unnecessary API calls.
pub async fn fetch_all(
    client: &CloudflareClient,
    state: &AppState,
    rd: &mut State<String>,
    st: &mut State<String>,
) {
    // Fetch zone name for display in title
    match client.get_zone_name().await {
        Ok(name) => {
            *state.zone_name.lock().unwrap() = name;
        }
        Err(_) => {
            // Keep default (zone ID) if fetch fails
        }
    }

    // Check cache first
    {
        let cache = state.dns_cache.lock().unwrap();
        if let Some(cached_records) = cache.get() {
            let cached_records = cached_records.clone();
            rd.set(format_records(&cached_records));
            st.set(format!(
                "Loaded {} DNS records (cached)",
                cached_records.len()
            ));
            *state.existing_ips.lock().unwrap() = extract_unique_ips(&cached_records);
            *state.records.lock().unwrap() = cached_records;
            return;
        }
    }

    // Cache miss — fetch from API
    match client.list_dns_records().await {
        Ok(f) => {
            rd.set(format_records(&f));
            st.set(format!("Loaded {} DNS records", f.len()));
            *state.existing_ips.lock().unwrap() = extract_unique_ips(&f);
            *state.records.lock().unwrap() = f.clone();
            state.dns_cache.lock().unwrap().set(f);
        }
        Err(e) => {
            rd.set(format!("Error: {}", e));
            st.set(format!("Error: {}", e));
        }
    }
}

/// Refresh DNS records only (used for manual refresh).
///
/// Invalidates the cache before fetching fresh records from the API.
pub async fn refresh_task(
    client: &CloudflareClient,
    state: &AppState,
    mut rd: State<String>,
    mut st: State<String>,
) {
    // Invalidate cache before refreshing
    state.dns_cache.lock().unwrap().invalidate();

    match client.list_dns_records().await {
        Ok(f) => {
            rd.set(format_records(&f));
            st.set(format!("Loaded {} DNS records", f.len()));
            *state.existing_ips.lock().unwrap() = extract_unique_ips(&f);
            *state.records.lock().unwrap() = f.clone();
            state.dns_cache.lock().unwrap().set(f);
        }
        Err(e) => st.set(format!("Error: {}", e)),
    }
}
