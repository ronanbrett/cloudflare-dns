use crate::colors::*;
use crate::components::app_layout::{AppLayoutConfig, render_app_layout};
use crate::components::form_field::FormField;
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
}

impl Default for CreateFormProps {
    fn default() -> Self {
        Self {
            form_type: None,
            form_name: None,
            form_content: None,
            form_ttl: None,
            form_proxied: None,
            form_focus: 0,
            status: String::new(),
            title: " Create DNS Record ".to_string(),
            hint: "Tab: navigate | Space on IP: selector | Enter: submit | Esc: cancel".to_string(),
            submit_label: "Submit".to_string(),
        }
    }
}

#[component]
pub fn CreateForm(props: &CreateFormProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let focus = props.form_focus;
    let title = props.title.clone();
    let hint = props.hint.clone();
    let form_type = props.form_type.clone();
    let form_name = props.form_name.clone();
    let form_content = props.form_content.clone();
    let form_ttl = props.form_ttl.clone();
    let form_proxied = props.form_proxied.clone();
    let submit_label = props.submit_label.clone();

    let content = element! {
        View(flex_grow: 1.0, align_items: AlignItems::Center, justify_content: JustifyContent::Center) {
            View(
                border_style: BorderStyle::Round,
                border_color: GREEN,
                background_color: MANTEL,
                padding_left: 3,
                padding_right: 3,
                padding_top: 2,
                padding_bottom: 2,
                width: 65,
            ) {
                View(flex_direction: FlexDirection::Column) {
                    View(margin_bottom: 1) {
                        Text(content: title.clone(), color: GREEN, weight: Weight::Bold)
                    }
                    View(margin_bottom: 1) {
                        Text(content: hint, color: OVERLAY1)
                    }
                    FormField(label: "Type", value: form_type, has_focus: focus == 0)
                    View(height: 1) { Text(content: "  (Space to cycle: A, AAAA, CNAME, MX...)", color: OVERLAY1) }
                    FormField(label: "Name", value: form_name, has_focus: focus == 1)
                    FormField(label: "IP Address", value: form_content, has_focus: focus == 2)
                    View(height: 1) { Text(content: "  (Space: open selector | Type: enter new IP)", color: OVERLAY1) }
                    FormField(label: "TTL", value: form_ttl, has_focus: focus == 3)
                    View(height: 1) { Text(content: "  (1 = auto)", color: OVERLAY1) }
                    FormField(label: "Proxied", value: form_proxied, has_focus: focus == 4)
                    View(height: 1) { Text(content: "  (Space to toggle)", color: OVERLAY1) }
                    View(
                        margin_top: 1,
                        border_style: if focus == 5 { BorderStyle::Round } else { BorderStyle::None },
                        border_color: GREEN,
                        padding: if focus == 5 { 0 } else { 1 },
                        align_items: AlignItems::Center,
                    ) {
                        Text(content: submit_label, color: TEXT, weight: Weight::Bold)
                    }
                }
            }
        }
    }.into_any();

    render_app_layout(
        AppLayoutConfig {
            title: title,
            menu: " [R]efresh  [C]reate  [E]dit  [Q]uit".to_string(),
            menu_bg: SURFACE1,
            status: props.status.clone(),
            ..Default::default()
        },
        content,
        &mut hooks,
    )
}
