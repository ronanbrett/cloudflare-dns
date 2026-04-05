use iocraft::prelude::*;
use std::sync::Arc;

use crate::constants::RECORD_TYPES;
use crate::state::{AppState, AppView};
use crate::status::StatusMessage;
use crate::tasks::*;

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

    // List selection
    pub list_sel_idx: State<usize>,
    pub is_deleting: State<bool>,

    // Global
    pub records_display: State<String>,
    pub status: State<String>,
    pub state: Arc<AppState>,
}

pub fn use_app_events(hooks: &mut Hooks<'_, '_>, ctx: &AppCtx) {
    hooks.use_future({
        let mut st = ctx.status.clone();
        async move {
            loop {
                let val = st.to_string();
                if val.is_empty() {
                    smol::Timer::after(std::time::Duration::from_millis(500)).await;
                    continue;
                }

                let is_transient = StatusMessage::is_transient(&val);

                if is_transient {
                    smol::Timer::after(std::time::Duration::from_secs(3)).await;
                    if st.to_string() == val {
                        st.set("".to_string());
                    }
                } else {
                    smol::Timer::after(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    });

    // ── Fetch on mount ──────────────────────────────────────────────────
    hooks.use_future({
        let state = ctx.state.clone();
        let mut rd = ctx.records_display.clone();
        let mut st = ctx.status.clone();
        async move {
            fetch_all(&state, &mut rd, &mut st).await;
        }
    });

    // ── Global keys (Q / C) ─────────────────────────────────────────────
    hooks.use_terminal_events({
        let mut should_exit = ctx.should_exit.clone();
        let mut view = ctx.view.clone();
        let mut ff = ctx.form_focus.clone();
        let mut ft = ctx.form_type.clone();
        let mut fn_ = ctx.form_name.clone();
        let mut fc = ctx.form_content.clone();
        let mut ftl = ctx.form_ttl.clone();
        let mut fp = ctx.form_proxied.clone();
        let mut eid = ctx.editing_record_id.clone();
        move |event| {
            if let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event {
                if kind == KeyEventKind::Release {
                    return;
                }
                if code == KeyCode::Char('q') || code == KeyCode::Char('Q') {
                    if view.get() == AppView::List {
                        should_exit.set(true);
                    }
                }
                if (code == KeyCode::Char('c') || code == KeyCode::Char('C'))
                    && view.get() == AppView::List
                {
                    view.set(AppView::Create);
                    eid.set("".to_string());
                    ff.set(0);
                    ft.set("A".to_string());
                    fn_.set("".to_string());
                    fc.set("".to_string());
                    ftl.set("1".to_string());
                    fp.set("false".to_string());
                }
            }
        }
    });

    // ── List keys (↑↓ R D E) ────────────────────────────────────────────
    hooks.use_terminal_events({
        let state = ctx.state.clone();
        let mut lsi = ctx.list_sel_idx.clone();
        let mut view = ctx.view.clone();
        let mut eid = ctx.editing_record_id.clone();
        let mut ft = ctx.form_type.clone();
        let mut fn_ = ctx.form_name.clone();
        let mut fc = ctx.form_content.clone();
        let mut ftl = ctx.form_ttl.clone();
        let mut fp = ctx.form_proxied.clone();
        let mut ff = ctx.form_focus.clone();
        let rd = ctx.records_display.clone();
        let mut st = ctx.status.clone();
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
                        let recs = state.records.lock().unwrap().clone();
                        if !recs.is_empty() {
                            let idx = lsi.get();
                            lsi.set(if idx > 0 { idx - 1 } else { recs.len() - 1 });
                        }
                    }
                    KeyCode::Down => {
                        let recs = state.records.lock().unwrap().clone();
                        if !recs.is_empty() {
                            let idx = lsi.get();
                            lsi.set(if idx < recs.len() - 1 { idx + 1 } else { 0 });
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        let (state, rd, mut st) = (state.clone(), rd.clone(), st.clone());
                        st.set("Refreshing...".to_string());
                        smol::spawn(refresh_task(state, rd, st)).detach();
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') => {
                        let recs = state.records.lock().unwrap().clone();
                        if !recs.is_empty() {
                            view.set(AppView::Delete);
                            st.set("Enter: confirm | Esc: cancel".to_string());
                        }
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        let recs = state.records.lock().unwrap().clone();
                        let idx = lsi.get();
                        if idx < recs.len() {
                            let rec = &recs[idx];
                            let edit_id = rec.id.clone().unwrap_or_default();
                            eid.set(edit_id.clone());
                            ff.set(0);
                            fill_form_from_record(
                                rec, &mut ft, &mut fn_, &mut fc, &mut ftl, &mut fp, &mut eid,
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
        let mut view = ctx.view.clone();
        let lsi = ctx.list_sel_idx.clone();
        let mut st = ctx.status.clone();
        let rd = ctx.records_display.clone();
        let is_del = ctx.is_deleting.clone();
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
                        let (state, lsi, view, mut is_del, st, rd) = (
                            state.clone(),
                            lsi.clone(),
                            view.clone(),
                            is_del.clone(),
                            st.clone(),
                            rd.clone(),
                        );
                        is_del.set(true);
                        smol::spawn(delete_task(state, lsi.get(), view, is_del, st, rd)).detach();
                    }
                    _ => {}
                }
            }
        }
    });

    // ── IP-selector keys ────────────────────────────────────────────────
    hooks.use_terminal_events({
        let state = ctx.state.clone();
        let mut view = ctx.view.clone();
        let mut isi = ctx.ip_sel_idx.clone();
        let mut ff = ctx.form_focus.clone();
        let mut fc = ctx.form_content.clone();
        let mut st = ctx.status.clone();
        let eid = ctx.editing_record_id.clone();
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
        let mut view = ctx.view.clone();
        let mut ff = ctx.form_focus.clone();
        let mut ft = ctx.form_type.clone();
        let fn_ = ctx.form_name.clone();
        let mut fc = ctx.form_content.clone();
        let ftl = ctx.form_ttl.clone();
        let mut fp = ctx.form_proxied.clone();
        let mut is = ctx.is_submitting.clone();
        let mut isi = ctx.ip_sel_idx.clone();
        let eid = ctx.editing_record_id.clone();
        let mut st = ctx.status.clone();
        let rd = ctx.records_display.clone();
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
                        view.set(AppView::List);
                        st.set("Cancelled".to_string());
                    }
                    KeyCode::Up => ff.set((ff.get() + 5) % 6),
                    KeyCode::Down => ff.set((ff.get() + 1) % 6),
                    KeyCode::Tab => ff.set((ff.get() + 1) % 6),
                    KeyCode::BackTab => ff.set((ff.get() + 5) % 6),
                    KeyCode::Enter if ff.get() == 5 && !is.get() => {
                        is.set(true);
                        let eid_str = eid.to_string();
                        let (state, rt, nm, ct, ttl, px, rd, st, view, fn_, fc, is) = (
                            state.clone(),
                            ft.to_string(),
                            fn_.to_string(),
                            fc.to_string(),
                            ftl.to_string(),
                            fp.to_string(),
                            rd.clone(),
                            st.clone(),
                            view.clone(),
                            fn_.clone(),
                            fc.clone(),
                            is.clone(),
                        );
                        smol::spawn(submit_task(
                            state, eid_str, rt, nm, ct, ttl, px, rd, st, view, fn_, fc, is,
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
                        isi.set(0);
                    }
                    _ => {}
                }
            }
        }
    });
}
