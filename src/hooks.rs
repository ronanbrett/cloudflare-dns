use iocraft::prelude::*;
use std::sync::Arc;

use crate::constants::RECORD_TYPES;
use crate::state::{AppState, AppView};
use crate::status::StatusMessage;
use crate::tasks::*;

/// Number of focusable items in the create/edit form.
const FORM_FIELD_COUNT: usize = 6;

#[derive(Clone)]
pub struct AppCtx {
    pub view: State<AppView>,
    pub should_exit: State<bool>,

    // Form fields
    pub form_focus: State<i32>,
    pub form_type: State<String>,
    pub form_name: State<String>,
    pub form_content: State<String>,
    pub form_ttl: State<String>,
    pub form_proxied: State<String>,
    pub is_submitting: State<bool>,
    pub editing_record_id: State<String>,

    // IP selector
    pub ip_sel_idx: State<usize>,
    pub ip_sel_open: State<bool>,

    // List selection
    pub list_sel_idx: State<usize>,
    pub is_deleting: State<bool>,

    // Refresh guard
    pub is_refreshing: State<bool>,

    // Global
    pub records_display: State<String>,
    pub status: State<String>,
    pub state: Arc<AppState>,
}

pub fn use_app_events(hooks: &mut Hooks<'_, '_>, ctx: &AppCtx) {
    // ── Status auto-clear via one-shot timers ────────────────────────────
    hooks.use_future({
        let mut st = ctx.status;
        async move {
            loop {
                #[allow(clippy::cmp_owned)]
                let val = st.to_string();
                if val.is_empty() || !StatusMessage::is_transient(&val) {
                    smol::Timer::after(std::time::Duration::from_secs(5)).await;
                    continue;
                }

                // Transient status — wait 3s then clear if unchanged
                smol::Timer::after(std::time::Duration::from_secs(3)).await;
                #[allow(clippy::cmp_owned)]
                if st.to_string() == val {
                    st.set("".to_string());
                }
            }
        }
    });

    // ── Fetch on mount ──────────────────────────────────────────────────
    hooks.use_future({
        let state = ctx.state.clone();
        let mut rd = ctx.records_display;
        let mut st = ctx.status;
        async move {
            fetch_all(&state, &mut rd, &mut st).await;
        }
    });

    // ── Global keys (Q / C) ─────────────────────────────────────────────
    hooks.use_terminal_events({
        let mut should_exit = ctx.should_exit;
        let mut view = ctx.view;
        let mut ff = ctx.form_focus;
        let mut ft = ctx.form_type;
        let mut form_name = ctx.form_name;
        let mut form_content = ctx.form_content;
        let mut ftl = ctx.form_ttl;
        let mut fp = ctx.form_proxied;
        let mut eid = ctx.editing_record_id;
        move |event| {
            if let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event {
                if kind == KeyEventKind::Release {
                    return;
                }
                if (code == KeyCode::Char('q') || code == KeyCode::Char('Q'))
                    && view.get() == AppView::List
                {
                    should_exit.set(true);
                }
                if (code == KeyCode::Char('c') || code == KeyCode::Char('C'))
                    && view.get() == AppView::List
                {
                    view.set(AppView::Create);
                    eid.set("".to_string());
                    ff.set(0);
                    ft.set("A".to_string());
                    form_name.set("".to_string());
                    form_content.set("".to_string());
                    ftl.set("1".to_string());
                    fp.set("false".to_string());
                }
            }
        }
    });

    // ── List keys (↑↓ R D E) ────────────────────────────────────────────
    hooks.use_terminal_events({
        let state = ctx.state.clone();
        let mut lsi = ctx.list_sel_idx;
        let mut view = ctx.view;
        let mut eid = ctx.editing_record_id;
        let mut ft = ctx.form_type;
        let mut form_name = ctx.form_name;
        let mut form_content = ctx.form_content;
        let mut ftl = ctx.form_ttl;
        let mut fp = ctx.form_proxied;
        let mut ff = ctx.form_focus;
        let rd = ctx.records_display;
        let mut st = ctx.status;
        let mut is_refreshing = ctx.is_refreshing;
        move |event| {
            if view.get() != AppView::List {
                return;
            }
            if let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event {
                if kind == KeyEventKind::Release {
                    return;
                }
                match code {
                    KeyCode::Up => {
                        let recs = state.records.lock().unwrap();
                        let len = recs.len();
                        if len > 0 {
                            let idx = lsi.get();
                            drop(recs);
                            lsi.set(if idx > 0 { idx - 1 } else { len - 1 });
                        }
                    }
                    KeyCode::Down => {
                        let recs = state.records.lock().unwrap();
                        let len = recs.len();
                        if len > 0 {
                            let idx = lsi.get();
                            drop(recs);
                            lsi.set(if idx < len - 1 { idx + 1 } else { 0 });
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        if is_refreshing.get() {
                            return;
                        }
                        is_refreshing.set(true);
                        let (state, rd, mut st, mut is_refreshing) =
                            (state.clone(), rd, st, is_refreshing);
                        smol::spawn(async move {
                            st.set("Refreshing...".to_string());
                            refresh_task(state.clone(), rd, st).await;
                            is_refreshing.set(false);
                        })
                        .detach();
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        let recs = state.records.lock().unwrap();
                        let idx = lsi.get();
                        if idx < recs.len() {
                            view.set(AppView::Delete);
                            st.set("Enter: confirm | Esc: cancel".to_string());
                        }
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        let recs = state.records.lock().unwrap();
                        let idx = lsi.get();
                        if idx < recs.len() {
                            let rec = &recs[idx];
                            let edit_id = rec.id.clone().unwrap_or_default();
                            eid.set(edit_id.clone());
                            ff.set(0);
                            fill_form_from_record(
                                rec,
                                &mut ft,
                                &mut form_name,
                                &mut form_content,
                                &mut ftl,
                                &mut fp,
                                &mut eid,
                            );
                            view.set(AppView::Edit);
                            st.set(format!(
                                "Editing {} ({}) — Esc to cancel",
                                rec.name, rec.record_type
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    // ── Delete-confirm keys ─────────────────────────────────────────────
    hooks.use_terminal_events({
        let state = ctx.state.clone();
        let mut view = ctx.view;
        let lsi = ctx.list_sel_idx;
        let mut st = ctx.status;
        let rd = ctx.records_display;
        let is_del = ctx.is_deleting;
        move |event| {
            if view.get() != AppView::Delete {
                return;
            }
            if let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event {
                if kind == KeyEventKind::Release {
                    return;
                }
                match code {
                    KeyCode::Esc => {
                        view.set(AppView::List);
                        st.set("Cancelled".to_string());
                    }
                    KeyCode::Enter if !is_del.get() => {
                        let recs = state.records.lock().unwrap();
                        let idx = lsi.get();
                        if idx >= recs.len() {
                            st.set("Record no longer exists".to_string());
                            view.set(AppView::List);
                            return;
                        }
                        let rec = &recs[idx];
                        let record_id = rec.id.clone();
                        let record_name = rec.name.clone();
                        let record_type = rec.record_type.clone();
                        drop(recs);

                        if record_id.is_none() {
                            st.set("No record ID".to_string());
                            view.set(AppView::List);
                            return;
                        }

                        let (state, record_id, record_name, record_type, view, mut is_del, st, rd) = (
                            state.clone(),
                            record_id.unwrap(),
                            record_name,
                            record_type,
                            view,
                            is_del,
                            st,
                            rd,
                        );
                        is_del.set(true);
                        smol::spawn(delete_task(
                            state, record_id, record_name, record_type, view, is_del, st, rd,
                        ))
                        .detach();
                    }
                    _ => {}
                }
            }
        }
    });

    // ── IP-selector keys ────────────────────────────────────────────────
    hooks.use_terminal_events({
        let state = ctx.state.clone();
        let mut view = ctx.view;
        let mut isi = ctx.ip_sel_idx;
        let mut ff = ctx.form_focus;
        let mut fc = ctx.form_content;
        let mut st = ctx.status;
        let eid = ctx.editing_record_id;
        move |event| {
            if view.get() != AppView::IpSelect {
                return;
            }
            if let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event {
                if kind == KeyEventKind::Release {
                    return;
                }
                match code {
                    KeyCode::Esc => {
                        let eid_str = eid.to_string();
                        view.set(if eid_str.is_empty() {
                            AppView::Create
                        } else {
                            AppView::Edit
                        });
                    }
                    KeyCode::Up => {
                        let ips = state.existing_ips.lock().unwrap().clone();
                        let len = ips.len() + 1;
                        let idx = isi.get();
                        isi.set(if idx > 0 { idx - 1 } else { len - 1 });
                    }
                    KeyCode::Down => {
                        let ips = state.existing_ips.lock().unwrap().clone();
                        let len = ips.len() + 1;
                        let idx = isi.get();
                        isi.set(if idx < len - 1 { idx + 1 } else { 0 });
                    }
                    KeyCode::Enter => {
                        let ips = state.existing_ips.lock().unwrap().clone();
                        let idx = isi.get();
                        if idx < ips.len() {
                            fc.set(ips[idx].clone());
                            st.set(format!("Selected: {}", ips[idx]));
                        } else {
                            fc.set("".to_string());
                            st.set("Type a new IP".to_string());
                        }
                        let eid_str = eid.to_string();
                        view.set(if eid_str.is_empty() {
                            AppView::Create
                        } else {
                            AppView::Edit
                        });
                        ff.set(2);
                    }
                    _ => {}
                }
            }
        }
    });

    // ── Create-form keys ────────────────────────────────────────────────
    hooks.use_terminal_events({
        let state = ctx.state.clone();
        let mut view = ctx.view;
        let mut ff = ctx.form_focus;
        let mut ft = ctx.form_type;
        let form_name = ctx.form_name;
        let fc = ctx.form_content;
        let ftl = ctx.form_ttl;
        let mut fp = ctx.form_proxied;
        let mut is = ctx.is_submitting;
        let mut isi = ctx.ip_sel_idx;
        let mut ip_sel_open = ctx.ip_sel_open;
        let eid = ctx.editing_record_id;
        let mut st = ctx.status;
        let rd = ctx.records_display;
        move |event| {
            if !matches!(view.get(), AppView::Create | AppView::Edit) {
                return;
            }
            if let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event {
                if kind == KeyEventKind::Release {
                    return;
                }
                match code {
                    KeyCode::Esc => {
                        // If IpSelect just handled Esc, don't double-process it.
                        if ip_sel_open.get() {
                            ip_sel_open.set(false);
                            return;
                        }
                        view.set(AppView::List);
                        st.set("Cancelled".to_string());
                    }
                    KeyCode::Up => {
                        ff.set((ff.get() + FORM_FIELD_COUNT as i32 - 1) % FORM_FIELD_COUNT as i32)
                    }
                    KeyCode::Down => ff.set((ff.get() + 1) % FORM_FIELD_COUNT as i32),
                    KeyCode::Tab => ff.set((ff.get() + 1) % FORM_FIELD_COUNT as i32),
                    KeyCode::BackTab => {
                        ff.set((ff.get() + FORM_FIELD_COUNT as i32 - 1) % FORM_FIELD_COUNT as i32)
                    }
                    KeyCode::Enter if ff.get() == 5 && !is.get() => {
                        // Client-side input validation before submit
                        let nm = form_name.to_string();
                        let ct = fc.to_string();
                        let ttl_str = ftl.to_string();
                        if nm.is_empty() {
                            st.set("Name cannot be empty".to_string());
                            return;
                        }
                        if ct.is_empty() {
                            st.set("Content cannot be empty".to_string());
                            return;
                        }
                        let ttl: i64 = match ttl_str.parse() {
                            Ok(v) => v,
                            Err(_) => {
                                st.set(format!("Invalid TTL '{}': must be a number", ttl_str));
                                return;
                            }
                        };
                        let px = fp.to_string().to_lowercase() == "true";
                        is.set(true);
                        let eid_str = eid.to_string();
                        let (state, rt, rd, st, view, form_name, fc, is) = (
                            state.clone(),
                            ft.to_string(),
                            rd,
                            st,
                            view,
                            form_name,
                            fc,
                            is,
                        );
                        smol::spawn(submit_task(
                            state, eid_str, rt, nm, ct, ttl, px, rd, st, view, form_name, fc, is,
                        ))
                        .detach();
                    }
                    KeyCode::Char(' ') if ff.get() == 0 => {
                        let c = ft.to_string();
                        let i = RECORD_TYPES.iter().position(|&t| t == c).unwrap_or(0);
                        ft.set(RECORD_TYPES[(i + 1) % RECORD_TYPES.len()].to_string());
                    }
                    KeyCode::Char(' ') if ff.get() == 4 => {
                        let c = fp.to_string().to_lowercase();
                        fp.set(if c == "true" {
                            "false".to_string()
                        } else {
                            "true".to_string()
                        });
                    }
                    KeyCode::Char(' ') if ff.get() == 2 => {
                        view.set(AppView::IpSelect);
                        ip_sel_open.set(true);
                        isi.set(0);
                    }
                    _ => {}
                }
            }
        }
    });
}
