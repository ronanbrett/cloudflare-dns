/// Submit (create/update) DNS record task with parameter object.
use iocraft::prelude::*;
use std::sync::Arc;

use crate::api::CloudflareClient;
use crate::api::models::DnsRecord;
use crate::ui::state::{AppState, AppView};
use crate::utils::{extract_unique_ips, format_records};

/// Parameters for the submit task.
pub struct SubmitParams {
    pub client: Arc<CloudflareClient>,
    pub state: Arc<AppState>,
    pub record_id: String,
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: i64,
    pub proxied: bool,
    pub records_display: State<String>,
    pub status: State<String>,
    pub view: State<AppView>,
    pub form_name: State<String>,
    pub form_content: State<String>,
    pub is_submitting: State<bool>,
}

/// Create or update a DNS record based on form data.
pub async fn submit_task(mut params: SubmitParams) {
    let is_update = !params.record_id.is_empty();
    let rec = DnsRecord {
        id: if is_update {
            Some(params.record_id.clone())
        } else {
            None
        },
        record_type: params.record_type.clone(),
        name: params.name.clone(),
        content: params.content.clone(),
        ttl: Some(params.ttl),
        proxied: Some(params.proxied),
        comment: None,
    };

    let result = if is_update {
        params.client.update_dns_record(&rec).await
    } else {
        params.client.create_dns_record(&rec).await
    };

    match result {
        Ok(_) => {
            let action = if is_update { "Updated" } else { "Created" };
            params.status.set(format!(
                "{} {} for {}",
                action, params.record_type, params.name
            ));
            params.view.set(AppView::List);
            params.form_name.set("".to_string());
            params.form_content.set("".to_string());
            params.is_submitting.set(false);

            if let Ok(f) = params.client.list_dns_records().await {
                params.records_display.set(format_records(&f));
                *params.state.existing_ips.lock().unwrap() = extract_unique_ips(&f);
                *params.state.records.lock().unwrap() = f;
            } else {
                params.status.set(format!(
                    "{} {} for {}, but refresh failed — press R to reload",
                    action, params.record_type, params.name
                ));
            }
        }
        Err(e) => {
            params.status.set(format!("Failed: {}", e));
            params.is_submitting.set(false);
        }
    }
}
