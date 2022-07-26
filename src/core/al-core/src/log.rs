use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

// ----------------------------------------------------------------------------
// Helpers to hide some of the verbosity of web_sys

/// Log some text to the developer console (`console.log(…)` in JS)
pub fn console_log(s: impl Into<JsValue>) {
    web_sys::console::log_1(&s.into());
}

/// Log a warning to the developer console (`console.warn(…)` in JS)
pub fn console_warn(s: impl Into<JsValue>) {
    web_sys::console::warn_1(&s.into());
}

/// Log an error to the developer console (`console.error(…)` in JS)
pub fn console_error(s: impl Into<JsValue>) {
    web_sys::console::error_1(&s.into());
}

#[macro_export]
macro_rules! log {
    // The pattern for a single `eval`
    ($($arg:tt)*) => {
        $( self::log(&format!("{:?}", $arg)); )*
    };
}

#[macro_export]
macro_rules! inforec {
    // The pattern for a single `eval`
    // Base case:
    ($x:expr) => (format!("{:?}", $x));
    // `$x` followed by at least one `$y,`
    ($x:expr, $($y:expr),+) => {
        // Call `find_min!` on the tail `$y`
        ( format!( "{} {}", inforec!($x), inforec!($($y),+) ) );
    }
}

#[macro_export]
macro_rules! info {
    // The pattern for a single `eval`
    ($($arg:expr),*) => {
        self::log( &inforec!( $( $arg ),* ) );
    };
}
