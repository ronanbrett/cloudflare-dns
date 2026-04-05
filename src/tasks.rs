use iocraft::prelude::*;
use std::sync::Arc;

use crate::cloudflare::DnsRecord;
use crate::state::{AppState, AppView};
use crate::utils::{extract_unique_ips, format_records};

pub async fn fetch_all(state: &AppState, rd: &mut State<String>, st: &mut State<String>) {
    // Fetch zone name for display in title
    match state.client.get_zone_name().await {
        Ok(name) => {
            *state.zone_name.lock().unwrap() = name;
        }
        Err(_) => {
            // Keep default (zone ID) if fetch fails
        }
    }

    match state.client.list_dns_records().await {
        Ok(f) => {
            rd.set(format_records(&f));
            st.set(format!("Loaded {} DNS records", f.len()));
            *state.existing_ips.lock().unwrap() = extract_unique_ips(&f);
            *state.records.lock().unwrap() = f;
        }
        Err(e) => {
            rd.set(format!("Error: {}", e));
            st.set(format!("Error: {}", e));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn fill_form_from_record(
    rec: &DnsRecord,
    form_type: &mut State<String>,
    form_name: &mut State<String>,
    form_content: &mut State<String>,
    form_ttl: &mut State<String>,
    form_proxied: &mut State<String>,
    editing_id: &mut State<String>,
    domain_suffix: &str,
) {
    form_type.set(rec.record_type.clone());
    // Strip the domain suffix from the name (e.g., "pihole.robrett.com" -> "pihole")
    let short_name = rec
        .name
        .strip_suffix(domain_suffix)
        .unwrap_or(&rec.name);
    form_name.set(short_name.to_string());
    form_content.set(rec.content.clone());
    form_ttl.set(rec.ttl.unwrap_or(1).to_string());
    form_proxied.set(
        if rec.proxied.unwrap_or(false) {
            "true"
        } else {
            "false"
        }
        .to_string(),
    );
    editing_id.set(rec.id.clone().unwrap_or_default());
}

pub async fn refresh_task(state: Arc<AppState>, mut rd: State<String>, mut st: State<String>) {
    match state.client.list_dns_records().await {
        Ok(f) => {
            rd.set(format_records(&f));
            st.set(format!("Loaded {} DNS records", f.len()));
            *state.existing_ips.lock().unwrap() = extract_unique_ips(&f);
            *state.records.lock().unwrap() = f;
        }
        Err(e) => st.set(format!("Error: {}", e)),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn delete_task(
    state: Arc<AppState>,
    record_id: String,
    record_name: String,
    record_type: String,
    mut view: State<AppView>,
    mut is_del: State<bool>,
    mut st: State<String>,
    mut rd: State<String>,
) {
    match state.client.delete_dns_record(&record_id).await {
        Ok(_) => {
            st.set(format!("Deleted {} ({})", record_name, record_type));
            view.set(AppView::List);
            is_del.set(false);
            if let Ok(f) = state.client.list_dns_records().await {
                rd.set(format_records(&f));
                *state.existing_ips.lock().unwrap() = extract_unique_ips(&f);
                *state.records.lock().unwrap() = f;
            } else {
                st.set(format!(
                    "Deleted {} ({}), but refresh failed — press R to reload",
                    record_name, record_type
                ));
            }
        }
        Err(e) => {
            st.set(format!("Failed: {}", e));
            is_del.set(false);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn submit_task(
    state: Arc<AppState>,
    eid: String,
    rt: String,
    nm: String,
    ct: String,
    ttl: i64,
    px: bool,
    mut rd: State<String>,
    mut st: State<String>,
    mut view: State<AppView>,
    mut fn_: State<String>,
    mut fc: State<String>,
    mut is: State<bool>,
) {
    let is_update = !eid.is_empty();
    let rec = DnsRecord {
        id: if is_update { Some(eid.clone()) } else { None },
        record_type: rt.clone(),
        name: nm.clone(),
        content: ct.clone(),
        ttl: Some(ttl),
        proxied: Some(px),
        comment: None,
    };
    let result = if is_update {
        state.client.update_dns_record(&rec).await
    } else {
        state.client.create_dns_record(&rec).await
    };
    match result {
        Ok(_) => {
            let action = if is_update { "Updated" } else { "Created" };
            st.set(format!("{} {} for {}", action, rt, nm));
            view.set(AppView::List);
            fn_.set("".to_string());
            fc.set("".to_string());
            is.set(false);
            if let Ok(f) = state.client.list_dns_records().await {
                rd.set(format_records(&f));
                *state.existing_ips.lock().unwrap() = extract_unique_ips(&f);
                *state.records.lock().unwrap() = f;
            } else {
                st.set(format!(
                    "{} {} for {}, but refresh failed — press R to reload",
                    action, rt, nm
                ));
            }
        }
        Err(e) => {
            st.set(format!("Failed: {}", e));
            is.set(false);
        }
    }
}
