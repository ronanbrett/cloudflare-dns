use crate::cloudflare::DnsRecord;
use crate::colors::*;
use crate::components::app_layout::{render_app_layout, AppLayoutConfig};
use iocraft::prelude::*;

#[derive(Props)]
pub struct RecordListProps {
    pub records: Vec<DnsRecord>,
    pub selected_idx: i32,
    pub status: String,
}

impl Default for RecordListProps {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            selected_idx: 0,
            status: String::new(),
        }
    }
}

#[component]
pub fn RecordList(props: &RecordListProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let sel = props.selected_idx as usize;
    let rec_text = if props.records.is_empty() {
        "No DNS records found\n\n↑↓: navigate | [D]elete | [C]reate".to_string()
    } else {
        let mut t = format!(
            "{} DNS Records  (↑↓: navigate, D: delete, R: refresh)\n\n",
            props.records.len()
        );
        for (i, r) in props.records.iter().enumerate() {
            let marker = if i == sel { "▸ " } else { "  " };
            t.push_str(&format!(
                "{}{:<6} │ {:<30} │ {:<20} │ TTL: {:<6} │ Proxy: {}\n",
                marker,
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
    };

    let content = element! {
        View(flex_grow: 1.0, padding_left: 2, padding_right: 2, padding_top: 1, padding_bottom: 1) {
            Text(content: rec_text, color: TEXT)
        }
    }.into_any();

    render_app_layout(
        AppLayoutConfig {
            menu: " [R]efresh  [C]reate  [E]dit  [D]elete  [Q]uit".to_string(),
            status: props.status.clone(),
            ..Default::default()
        },
        content,
        &mut hooks,
    )
}
