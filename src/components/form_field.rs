use crate::colors::{BLUE, OVERLAY1, SURFACE0, TEAL};
use crate::state::FormFieldProps;
use iocraft::prelude::*;

#[component]
pub fn FormField(props: &FormFieldProps, _hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let Some(mut value) = props.value else {
        // Graceful fallback: render a disabled-looking field
        return element! {
            View(flex_direction: FlexDirection::Row, margin_bottom: 1, align_items: AlignItems::Center) {
                View(width: 12) { Text(content: format!("{}: ", props.label), color: TEAL) }
                View(
                    border_style: BorderStyle::None,
                    padding: 1,
                    width: 40,
                    background_color: SURFACE0,
                ) {
                    Text(content: "(no value)", color: OVERLAY1)
                }
            }
        }
    };

    element! {
        View(flex_direction: FlexDirection::Row, margin_bottom: 1, align_items: AlignItems::Center) {
            View(width: 12) {
                Text(content: format!("{}: ", props.label), color: TEAL)
            }
            View(
                border_style: if props.has_focus { BorderStyle::Round } else { BorderStyle::None },
                border_color: BLUE,
                padding: if props.has_focus { 0 } else { 1 },
                width: 40,
                background_color: SURFACE0,
            ) {
                TextInput(
                    has_focus: props.has_focus,
                    value: value.to_string(),
                    on_change: move |new_value| value.set(new_value),
                )
            }
            View(margin_left: 1) {
                Text(content: props.suffix.clone(), color: OVERLAY1)
            }
        }
    }
}
