use crate::colors::*;
use crate::components::app_layout::{render_app_layout, AppLayoutConfig};
use iocraft::prelude::*;

#[derive(Props)]
pub struct IpSelectorProps {
    pub sel_text: String,
    pub status: String,
}

impl Default for IpSelectorProps {
    fn default() -> Self {
        Self {
            sel_text: String::new(),
            status: String::new(),
        }
    }
}

#[component]
pub fn IpSelector(props: &IpSelectorProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let sel_text = props.sel_text.clone();
    
    let content = element! {
        View(flex_grow: 1.0, align_items: AlignItems::Center, justify_content: JustifyContent::Center) {
            View(border_style: BorderStyle::Round, border_color: SAPPHIRE, background_color: MANTEL, padding_left: 1, padding_right: 1, padding_top: 2, padding_bottom: 2, width: 50) {
                View(flex_direction: FlexDirection::Column) {
                    View(margin_bottom: 1, padding_left: 1) { Text(content: " Select IP Address ", color: SAPPHIRE, weight: Weight::Bold) }
                    View(margin_bottom: 1, padding_left: 1) { Text(content: "↑↓: navigate | Enter: select | Esc: back", color: OVERLAY1) }
                    View(border_style: BorderStyle::Single, border_color: SURFACE1, padding_left: 1, padding_right: 1, padding_top: 1, padding_bottom: 1) { Text(content: sel_text, color: SUBTEXT0) }
                }
            }
        }
    }.into_any();

    render_app_layout(
        AppLayoutConfig {
            menu: " [R]efresh  [C]reate  [Q]uit".to_string(),
            status: props.status.clone(),
            ..Default::default()
        },
        content,
        &mut hooks,
    )
}
