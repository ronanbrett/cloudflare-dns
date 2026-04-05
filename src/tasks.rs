use iocraft::prelude::*;
use std::collections::BTreeSet;
use std::sync::Arc;

use crate::cloudflare::DnsRecord;
use crate::state::{AppState, AppView};
use crate::utils::format_records;

pub async fn fetch_all(state: &AppState, rd: &mut State<String>, st: &mut State<String>) {
    match state.client.list_dns_records().await {
        Ok(f) => {
            rd.set(format_records(&f));
            st.set(format!("Loaded {} DNS records", f.len()));
            let mut ips: BTreeSet<String> = BTreeSet::new();
            for r in &f {
                if !ips.contains(&r.content) && !r.content.ends_with('.') {
                    ips.insert(r.content.clone());
                }
            }
            *state.existing_ips.lock().unwrap() = ips.into_iter().collect();
            *state.records.lock().unwrap() = f;
        }
        Err(e) => {
            rd.set(format!("Error: {}", e));
            st.set(format!("Error: {}", e));
        }
    }
}

pub fn fill_form_from_record(
    rec: &DnsRecord,
    form_type: &mut State<String>,
    form_name: &mut State<String>,
    form_content: &mut State<String>,
    form_ttl: &mut State<String>,
    form_proxied: &mut State<String>,
    editing_id: &mut State<String>,
) {
    form_type.set(rec.record_type.clone());
    form_name.set(rec.name.clone());
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
            let mut ips = BTreeSet::new();
            for r in &f {
                if !ips.contains(&r.content) && !r.content.ends_with('.') {
                    ips.insert(r.content.clone());
                }
            }
            *state.existing_ips.lock().unwrap() = ips.into_iter().collect();
            *state.records.lock().unwrap() = f;
        }
        Err(e) => st.set(format!("Error: {}", e)),
    }
}

pub async fn delete_task(
    state: Arc<AppState>,
    lsi: usize,
    mut view: State<AppView>,
    mut is_del: State<bool>,
    mut st: State<String>,
    mut rd: State<String>,
) {
    let recs = state.records.lock().unwrap().clone();
    let idx = lsi as usize;
    if idx < recs.len() {
        let rec = &recs[idx];
        if let Some(ref rid) = rec.id {
            match state.client.delete_dns_record(rid).await {
                Ok(_) => {
                    st.set(format!("Deleted {} ({})", rec.name, rec.record_type));
                    view.set(AppView::List);
                    is_del.set(false);
                    if let Ok(f) = state.client.list_dns_records().await {
                        rd.set(format_records(&f));
                        *state.records.lock().unwrap() = f.clone();
                        let mut ips = BTreeSet::new();
                        for r in &f {
                            if !ips.contains(&r.content) && !r.content.ends_with('.') {
                                ips.insert(r.content.clone());
                            }
                        }
                        *state.existing_ips.lock().unwrap() = ips.into_iter().collect();
                    }
                }
                Err(e) => {
                    st.set(format!("Failed: {}", e));
                    is_del.set(false);
                }
            }
        } else {
            st.set("No record ID".to_string());
            is_del.set(false);
        }
    } else {
        st.set("Index out of range".to_string());
        is_del.set(false);
    }
}

pub async fn submit_task(
    state: Arc<AppState>,
    eid: String,
    rt: String,
    nm: String,
    ct: String,
    ttl: String,
    px: String,
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
        ttl: Some(ttl.parse().unwrap_or(1)),
        proxied: Some(px.to_lowercase() == "true"),
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
                *state.records.lock().unwrap() = f.clone();
                let mut ips = BTreeSet::new();
                for r in &f {
                    if !ips.contains(&r.content) && !r.content.ends_with('.') {
                        ips.insert(r.content.clone());
                    }
                }
                *state.existing_ips.lock().unwrap() = ips.into_iter().collect();
            }
        }
        Err(e) => {
            st.set(format!("Failed: {}", e));
            is.set(false);
        }
    }
}
