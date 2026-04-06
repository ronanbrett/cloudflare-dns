use crate::ui::colors::*;
use crate::ui::components::app_layout::{AppLayoutConfig, render_app_layout};
use crate::ui::components::form_field::FormField;
use iocraft::prelude::*;

#[derive(Props)]
pub struct CreateFormProps {
    pub form_type: Option<State<String>>,
    pub form_name: Option<State<String>>,
    pub form_content: Option<State<String>>,
    pub form_ttl: Option<State<String>>,
    pub form_proxied: Option<State<String>>,
    pub form_focus: i32,
    pub status: String,
    pub title: String,
    pub hint: String,
    pub submit_label: String,
    pub zone_name: String,
    pub domain_suffix: String,
    pub is_editing: bool,
}

impl Default for CreateFormProps {
    fn default() -> Self {
        Self {
            form_type: None,
            form_name: None,
            form_content: None,
            form_ttl: None,
            form_proxied: None,
            form_focus: 1,
            status: String::new(),
            title: " Create DNS Record ".to_string(),
            hint: "↑↓: navigate | esc: cancel".to_string(),
            submit_label: "Submit".to_string(),
            zone_name: String::new(),
            domain_suffix: String::new(),
            is_editing: false,
        }
    }
}

#[component]
pub fn CreateForm(props: &CreateFormProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let focus = props.form_focus;
    let title = format!(" {} — {} ", props.title.trim(), props.zone_name);
    let subtitle = props.title.trim();
    let hint = props.hint.clone();
    let form_type = props.form_type;
    let form_name = props.form_name;
    let form_content = props.form_content;
    let form_ttl = props.form_ttl;
    let form_proxied = props.form_proxied;
    let submit_label = props.submit_label.clone();
    let accent = if props.is_editing { SAPPHIRE } else { GREEN };

    let content = element! {
        View(flex_grow: 1.0, align_items: AlignItems::Center, justify_content: JustifyContent::Center) {
            View(
                border_style: BorderStyle::Round,
                border_color: accent,
                background_color: MANTEL,
                padding_left: 3,
                padding_right: 3,
                padding_top: 2,
                padding_bottom: 2,
                width: 80,
            ) {
                View(flex_direction: FlexDirection::Column) {
                    View(margin_bottom: 1) {
                        Text(content: subtitle, color: accent, weight: Weight::Bold)
                    }
                    View(margin_bottom: 1) {
                        Text(content: hint, color: OVERLAY1)
                    }
                    FormField(label: "Type", value: form_type, has_focus: focus == 0, is_editable: false)
                    FormField(label: "Name", value: form_name, has_focus: focus == 1, suffix: props.domain_suffix.clone(), is_editable: true)
                    FormField(label: "IP Address", value: form_content, has_focus: focus == 2, is_editable: true)
                    FormField(label: "TTL", value: form_ttl, has_focus: focus == 3, is_editable: true)
                    FormField(label: "Proxied", value: form_proxied, has_focus: focus == 4, is_editable: false)
                    View(
                        margin_top: 1,
                        border_style: if focus == 5 { BorderStyle::Round } else { BorderStyle::None },
                        border_color: accent,
                        padding: if focus == 5 { 0 } else { 1 },
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                    ) {
                        Text(content: submit_label, color: TEXT, weight: Weight::Bold, align: TextAlign::Center)
                    }
                }
            }
        }
    }.into_any();

    render_app_layout(
        AppLayoutConfig {
            title,
            title_bg: accent,
            menu: " ↑↓: navigate | space: toggle [Type] or [Proxied] | enter: submit | esc: cancel | q: quit"
                .to_string(),
            menu_bg: SURFACE1,
            status: props.status.clone(),
            ..Default::default()
        },
        content,
        &mut hooks,
    )
}
