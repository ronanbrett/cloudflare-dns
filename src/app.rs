use anyhow::Result;
use iocraft::prelude::*;
use std::sync::Arc;

use crate::components::create_form::CreateForm;
use crate::components::delete_confirm::DeleteConfirm;
use crate::components::ip_selector::IpSelector;
use crate::components::record_list::RecordList;
use crate::hooks::*;
use crate::state::{AppProps, AppState, AppView};
use crate::status::{generate_contextual_status, StatusMessage};
use crate::utils::format_selector;

// ─── App ────────────────────────────────────────────────────────────────────

#[component]
pub fn App(props: &AppProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (_width, _height) = hooks.use_terminal_size();
    let records_display = hooks.use_state(|| "Loading DNS records...".to_string());
    let status = hooks.use_state(|| "Initializing...".to_string());
    let should_exit = hooks.use_state(|| false);
    let mut system = hooks.use_context_mut::<SystemContext>();

    // View state
    let view = hooks.use_state(|| AppView::List);

    // Form fields
    let form_focus = hooks.use_state(|| 0);
    let form_type = hooks.use_state(|| "A".to_string());
    let form_name = hooks.use_state(|| "".to_string());
    let form_content = hooks.use_state(|| "".to_string());
    let form_ttl = hooks.use_state(|| "1".to_string());
    let form_proxied = hooks.use_state(|| "false".to_string());
    let is_submitting = hooks.use_state(|| false);
    let editing_record_id = hooks.use_state(|| String::new()); // empty = creating

    // IP selector
    let ip_sel_idx = hooks.use_state(|| 0);

    // List selection
    let list_sel_idx = hooks.use_state(|| 0);
    let is_deleting = hooks.use_state(|| false);

    // ── Context Encapsulation ───────────────────────────────────────────
    let ctx = AppCtx {
        view: view.clone(),
        should_exit: should_exit.clone(),
        form_focus: form_focus.clone(),
        form_type: form_type.clone(),
        form_name: form_name.clone(),
        form_content: form_content.clone(),
        form_ttl: form_ttl.clone(),
        form_proxied: form_proxied.clone(),
        is_submitting: is_submitting.clone(),
        editing_record_id: editing_record_id.clone(),
        ip_sel_idx: ip_sel_idx.clone(),
        list_sel_idx: list_sel_idx.clone(),
        is_deleting: is_deleting.clone(),
        records_display: records_display.clone(),
        status: status.clone(),
        state: props.state.clone(),
    };

    // ── Application Event Listeners ─────────────────────────────────────
    use_app_events(&mut hooks, &ctx);

    // ── Exit ────────────────────────────────────────────────────────────
    if should_exit.get() {
        system.exit();
    }

    // ── Snapshot ────────────────────────────────────────────────────────
    let records = props.state.records.lock().unwrap().clone();
    let ips = props.state.existing_ips.lock().unwrap().clone();
    let sel_text = format_selector(&ips, ip_sel_idx.get());
    let rec_name = if list_sel_idx.get() < records.len() {
        format!(
            "{} ({})",
            records[list_sel_idx.get()].name,
            records[list_sel_idx.get()].record_type
        )
    } else {
        "Unknown".to_string()
    };
    let editing = matches!(view.get(), AppView::Edit);
    let lsi = list_sel_idx.get();

    // ── Contextual status text ──────────────────────────────────────────
    let status_val = status.to_string();
    let is_transient = StatusMessage::is_transient(&status_val);
    
    let status_text = if is_transient {
        status_val
    } else {
        let editing = matches!(view.get(), AppView::Edit);
        let rec_name = if list_sel_idx.get() < records.len() {
            Some(records[list_sel_idx.get()].name.as_str())
        } else {
            None
        };
        
        let status_msg = generate_contextual_status(
            &view.get(),
            form_focus.get() as usize,
            &form_type.to_string(),
            &form_proxied.to_string(),
            editing,
            records.len(),
            list_sel_idx.get(),
            rec_name,
        );
        status_msg.render()
    };

    // ── Render ──────────────────────────────────────────────────────────
    match view.get() {
        AppView::Delete => element! {
            DeleteConfirm(rec_name: rec_name, deleting: is_deleting.get(), status: status_text)
        }
        .into_any(),
        AppView::IpSelect => element! {
            IpSelector(sel_text: sel_text, status: status_text)
        }
        .into_any(),
        AppView::Create | AppView::Edit => {
            let title = if editing {
                " Edit DNS Record "
            } else {
                " Create DNS Record "
            };
            let hint = if editing {
                "Tab: navigate | Space on IP: selector | Enter: save | Esc: cancel"
            } else {
                "Tab: navigate | Space on IP: selector | Enter: submit | Esc: cancel"
            };
            element! {
                CreateForm(
                    form_type: form_type,
                    form_name: form_name,
                    form_content: form_content,
                    form_ttl: form_ttl,
                    form_proxied: form_proxied,
                    form_focus: form_focus.get() as i32,
                    status: status_text,
                    title: title.to_string(),
                    hint: hint.to_string(),
                    submit_label: if editing { "Save" } else { "Submit" },
                )
            }
            .into_any()
        }
        AppView::List => element! {
            RecordList(
                records: records,
                selected_idx: lsi as i32,
                status: status_text,
            )
        }
        .into_any(),
    }
}

pub fn run_app(api_token: String, zone_id: String) -> Result<()> {
    let state = Arc::new(AppState::new(api_token, zone_id));
    smol::block_on(element!(App(state: state.clone())).fullscreen())?;
    Ok(())
}
