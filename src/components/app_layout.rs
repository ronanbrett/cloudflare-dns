use crate::colors::*;
use crate::components::status_bar::StatusBar;
use iocraft::prelude::*;

pub struct AppLayoutConfig {
    pub border_color: Color,
    pub title: String,
    pub title_bg: Color,
    pub title_color: Color,
    pub menu: String,
    pub menu_bg: Color,
    pub menu_color: Color,
    pub status: String,
}

impl Default for AppLayoutConfig {
    fn default() -> Self {
        Self {
            border_color: BLUE,
            title: " ☁ Cloudflare DNS ".to_string(),
            title_bg: BLUE,
            title_color: CRUST,
            menu: String::new(),
            menu_bg: SURFACE0,
            menu_color: SUBTEXT1,
            status: String::new(),
        }
    }
}

pub fn render_app_layout<'a>(
    config: AppLayoutConfig,
    content: AnyElement<'a>,
    hooks: &mut Hooks,
) -> AnyElement<'a> {
    let (width, height) = hooks.use_terminal_size();

    element! {
        View(width: width, height: height, background_color: CRUST, flex_direction: FlexDirection::Column) {
            View(background_color: config.title_bg, padding_left: 2, padding_right: 2, padding_top: 1, padding_bottom: 1) {
                Text(content: config.title, color: config.title_color, weight: Weight::Bold)
            }
            View(background_color: config.menu_bg, padding_left: 2, padding_right: 2, padding_top: 1, padding_bottom: 1) {
                Text(content: config.menu, color: config.menu_color)
            }
            #(content)
            StatusBar(message: config.status)
        }
    }.into_any()
}
