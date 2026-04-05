use crate::api::DnsRecord;
use crate::ui::colors::*;
use crate::ui::components::app_layout::{AppLayoutConfig, render_app_layout};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct RecordListProps {
    pub records: Vec<DnsRecord>,
    pub selected_idx: i32,
    pub status: String,
    pub zone_name: String,
}

#[component]
pub fn RecordList(props: &RecordListProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let sel = props.selected_idx as usize;
    let title = format!(" ☁ Cloudflare DNS — {} ", props.zone_name);
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
    }
    .into_any();

    render_app_layout(
        AppLayoutConfig {
            border_color: BLUE,
            title,
            title_bg: ORANGE,
            title_color: CRUST,
            menu: " [R]efresh  [C]reate  [E]dit  [D]elete  [Q]uit".to_string(),
            menu_bg: SURFACE1,
            menu_color: SUBTEXT1,
            status: props.status.clone(),
        },
        content,
        &mut hooks,
    )
}
