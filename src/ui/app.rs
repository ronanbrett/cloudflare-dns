/// Main application component and entry point.
///
/// This module defines the root App component and the run_app function
/// that initializes the TUI.
use anyhow::Result;
use iocraft::prelude::*;
use std::sync::Arc;

use crate::ui::components::create_form::CreateForm;
use crate::ui::components::delete_confirm::DeleteConfirm;
use crate::ui::components::ip_selector::IpSelector;
use crate::ui::components::record_list::RecordList;
use crate::ui::hooks::*;
use crate::ui::state::{AppProps, AppState, AppView};
use crate::ui::status::{StatusMessage, generate_contextual_status};
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
    // iocraft's use_state requires FnOnce, not a direct value
    #[allow(clippy::redundant_closure)]
    let editing_record_id = hooks.use_state(|| String::new()); // empty = creating

    // IP selector
    let ip_sel_idx = hooks.use_state(|| 0);
    let ip_sel_open = hooks.use_state(|| false);

    // List selection
    let list_sel_idx = hooks.use_state(|| 0);
    let is_deleting = hooks.use_state(|| false);

    // Refresh guard
    let is_refreshing = hooks.use_state(|| false);

    // ── Context Encapsulation ───────────────────────────────────────────
    let ctx = AppCtx {
        view,
        should_exit,
        form_focus,
        form_type,
        form_name,
        form_content,
        form_ttl,
        form_proxied,
        is_submitting,
        editing_record_id,
        ip_sel_idx,
        ip_sel_open,
        list_sel_idx,
        is_deleting,
        is_refreshing,
        records_display,
        status,
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
    let zone_name = props.state.zone_name.lock().unwrap().clone();
    let domain_suffix = format!(".{}", &zone_name);
    let sel_text = format_selector(&ips, ip_sel_idx.get());

    let lsi = list_sel_idx.get();
    let rec_name: String = if lsi < records.len() {
        format!("{} ({})", records[lsi].name, records[lsi].record_type)
    } else {
        "Unknown".to_string()
    };
    let is_editing = matches!(view.get(), AppView::Edit);

    // ── Contextual status text ──────────────────────────────────────────
    let status_val = status.to_string();
    let is_transient = StatusMessage::is_transient(&status_val);

    let status_text = if is_transient {
        status_val
    } else {
        let rec_name_opt = if lsi < records.len() {
            Some(records[lsi].name.as_str())
        } else {
            None
        };

        let status_msg = generate_contextual_status(
            &view.get(),
            form_focus.get() as usize,
            &form_type.to_string(),
            &form_proxied.to_string(),
            is_editing,
            records.len(),
            lsi,
            rec_name_opt,
        );
        status_msg.render()
    };

    // ── Render ──────────────────────────────────────────────────────────
    match view.get() {
        AppView::Delete => element! {
            DeleteConfirm(rec_name: rec_name, deleting: is_deleting.get(), status: status_text, zone_name: zone_name.clone())
        }
        .into_any(),
        AppView::IpSelect => element! {
            IpSelector(sel_text: sel_text, status: status_text, zone_name: zone_name.clone())
        }
        .into_any(),
        AppView::Create | AppView::Edit => {
            let title = if is_editing {
                " Edit DNS Record "
            } else {
                " Create DNS Record "
            };
            let hint = if is_editing {
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
                    form_focus: form_focus.get(),
                    status: status_text,
                    title: title.to_string(),
                    hint: hint.to_string(),
                    submit_label: if is_editing { "Save" } else { "Submit" },
                    zone_name: zone_name.clone(),
                    domain_suffix: domain_suffix.clone(),
                )
            }
            .into_any()
        }
        AppView::List => element! {
            RecordList(
                records: records,
                selected_idx: lsi as i32,
                status: status_text,
                zone_name: zone_name,
            )
        }
        .into_any(),
    }
}

/// Run the TUI application.
pub fn run_app(api_token: String, zone_id: String) -> Result<()> {
    let state = Arc::new(AppState::new(api_token, zone_id));
    smol::block_on(element!(App(state: state.clone())).fullscreen())?;
    Ok(())
}
