use crate::colors::{SURFACE0, YELLOW};
use crate::state::StatusBarProps;
use iocraft::prelude::*;

#[component]
pub fn StatusBar(props: &StatusBarProps, _hooks: Hooks) -> impl Into<AnyElement<'static>> {
    element! {
        View(background_color: SURFACE0, padding_left: 2, padding_right: 2, padding_top: 1, padding_bottom: 1) {
            Text(content: props.message.clone(), color: YELLOW)
        }
    }
}
