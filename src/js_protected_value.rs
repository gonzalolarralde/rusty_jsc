use std::ops::Deref;

use rusty_jsc_sys::{JSValueProtect, JSValueUnprotect};
use crate::{JSContext, JSValue};

pub struct JSProtectedValue(JSValue, JSContext);

impl JSProtectedValue {
    // TODO: Not making this public as this requires JSContext to be sync.
    // There's no good way yet to encode that guarantee so keeping it as
    // a convenience pattern for JSContext.
    pub(crate) fn new(context: &JSContext, value: JSValue) -> Self {
        unsafe { JSValueProtect(context.inner(), value.inner) };
        JSProtectedValue(value, context.clone())
    }
}

impl Drop for JSProtectedValue {
    fn drop(&mut self) {
        unsafe { JSValueUnprotect(self.1.inner(), self.0.inner) }
    }
}

impl Deref for JSProtectedValue {
    type Target = JSValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
