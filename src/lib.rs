//! This library provides a Rust API for the <a href="https://developer.apple.com/documentation/javascriptcore">JavaScriptCore</a> engine.
//!
//! # Example
//!
//! The JavaScriptCore engine lets you evaluate JavaScript scripts from your
//! own application. You first need to create a `JSContext` object and then
//! call its `evaluate_script` method.
//!
//! ```rust
//! use rusty_jsc::JSContext;
//!
//! let mut context = JSContext::default();
//! match context.evaluate_script("'hello, world'", 1) {
//!     Ok(value) => {
//!         println!("{}", value.to_string(&context).unwrap());
//!     }
//!     Err(e) => {
//!         println!(
//!             "Uncaught: {}",
//!             e.to_string(&context).unwrap()
//!         )
//!     }
//! }
//! ```

mod internal;

// #[macro_export]
mod closure;
pub use crate::internal::JSString;
pub use rusty_jsc_macros::callback;
pub use rusty_jsc_sys::JSObjectCallAsFunctionCallback;
pub mod private {
    pub use rusty_jsc_sys::*;
}

// pub use crate::closure::callback_closure;

mod js_vm;
pub use js_vm::*;

mod js_context;
pub use js_context::*;

mod js_value;
pub use js_value::*;

mod js_object;
pub use js_object::*;
