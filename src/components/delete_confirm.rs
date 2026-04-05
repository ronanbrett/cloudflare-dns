use crate::colors::*;
use crate::components::app_layout::{render_app_layout, AppLayoutConfig};
use iocraft::prelude::*;

#[derive(Props)]
pub struct DeleteConfirmProps {
    pub rec_name: String,
    pub deleting: bool,
    pub status: String,
}

impl Default for DeleteConfirmProps {
    fn default() -> Self {
        Self {
            rec_name: String::new(),
            deleting: false,
            status: String::new(),
        }
    }
}

#[component]
pub fn DeleteConfirm(
    props: &DeleteConfirmProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let rec_name = props.rec_name.clone();
    let deleting = props.deleting;

    let content = element! {
        View(flex_grow: 1.0, align_items: AlignItems::Center, justify_content: JustifyContent::Center) {
            View(border_style: BorderStyle::Round, border_color: RED, background_color: MANTEL, padding_left: 3, padding_right: 3, padding_top: 2, padding_bottom: 2, width: 55) {
                View(flex_direction: FlexDirection::Column) {
                    View(margin_bottom: 1) { Text(content: "Confirm Delete", color: RED, weight: Weight::Bold) }
                    View(margin_bottom: 1) { Text(content: format!("Record: {}", rec_name), color: TEXT) }
                    View(height: 1) { Text(content: "", color: CRUST) }
                    View(margin_top: 1) {
                        Text(
                            content: if deleting { "  Deleting..." } else { "  Enter: confirm | Esc: cancel" },
                            color: if deleting { OVERLAY1 } else { YELLOW },
                        )
                    }
                }
            }
        }
    }.into_any();

    render_app_layout(
        AppLayoutConfig {
            border_color: RED,
            title: " ☁ Cloudflare DNS ".to_string(),
            title_bg: SURFACE1,
            title_color: SUBTEXT1,
            menu: " ⚠ DELETE RECORD ".to_string(),
            menu_bg: RED,
            menu_color: CRUST,
            status: props.status.clone(),
            ..Default::default()
        },
        content,
        &mut hooks,
    )
}
