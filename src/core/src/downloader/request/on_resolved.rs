/// This stores globally a notifier function that wakes up the JS.
// It ultimately triggers a redraw of the scene.

use std::cell::RefCell;

thread_local! {
    static WAKE_UP_CB: RefCell<Option<js_sys::Function>> = RefCell::new(None);
}

pub fn set_on_resolved_cb(cb: js_sys::Function) {
    WAKE_UP_CB.with(|f| *f.borrow_mut() = Some(cb));
}

use crate::JsValue;
pub fn call_on_resolved_cb() {
    WAKE_UP_CB.with(|f| {
        if let Some(cb) = f.borrow().as_ref() {
            let _ = cb.call0(&JsValue::NULL);
        }
    });
}