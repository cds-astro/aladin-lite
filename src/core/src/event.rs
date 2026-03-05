use wasm_bindgen::prelude::*;
use web_sys::{window, CustomEvent, CustomEventInit};

pub(crate) fn send_custom_event(name: &str, value: JsValue) {
    // Create event details (optional)
    let mut event_init = CustomEventInit::new();
    event_init.detail(&value);

    // Create the event
    let event = CustomEvent::new_with_event_init_dict(name, &event_init)
        .expect("Failed to create custom event");

    // Dispatch the event on the window or any target element
    window()
        .expect("no global `window` exists")
        .dispatch_event(&event)
        .expect("failed to dispatch event");
}
