/// Delete DNS record task with parameter object.
use iocraft::prelude::*;
use std::sync::Arc;

use crate::api::CloudflareClient;
use crate::ui::state::{AppState, AppView};
use crate::utils::{extract_unique_ips, format_records};

/// Parameters for the delete task.
pub struct DeleteParams {
    pub client: Arc<CloudflareClient>,
    pub state: Arc<AppState>,
    pub record_id: String,
    pub record_name: String,
    pub record_type: String,
    pub view: State<AppView>,
    pub is_deleting: State<bool>,
    pub status: State<String>,
    pub records_display: State<String>,
}

/// Execute a DNS record deletion with automatic refresh on success.
pub async fn delete_task(mut params: DeleteParams) {
    match params.client.delete_dns_record(&params.record_id).await {
        Ok(_) => {
            // Invalidate cache after successful deletion
            params.state.dns_cache.lock().unwrap().invalidate();

            params.status.set(format!(
                "Deleted {} ({})",
                params.record_name, params.record_type
            ));
            params.view.set(AppView::List);
            params.is_deleting.set(false);

            if let Ok(f) = params.client.list_dns_records().await {
                params.records_display.set(format_records(&f));
                *params.state.existing_ips.lock().unwrap() = extract_unique_ips(&f);
                *params.state.records.lock().unwrap() = f.clone();
                params.state.dns_cache.lock().unwrap().set(f);
            } else {
                params.status.set(format!(
                    "Deleted {} ({}), but refresh failed — press R to reload",
                    params.record_name, params.record_type
                ));
            }
        }
        Err(e) => {
            params.status.set(format!("Failed: {}", e));
            params.is_deleting.set(false);
        }
    }
}
