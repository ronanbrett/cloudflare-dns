use crate::colors::{BLUE, SURFACE0, TEAL};
use crate::state::FormFieldProps;
use iocraft::prelude::*;

#[component]
pub fn FormField(props: &FormFieldProps, _hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let Some(mut value) = props.value else {
        panic!("value is required");
    };

    element! {
        View(flex_direction: FlexDirection::Row, margin_bottom: 1) {
            View(width: 12) { Text(content: format!("{}: ", props.label), color: TEAL) }
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
        }
    }
}
