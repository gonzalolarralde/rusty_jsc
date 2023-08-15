use crate::internal::JSString;
use rusty_jsc_sys::*;
use std::fmt;

use crate::helpers::RetainReleaseWrapper;

use crate::js_value::JSValue;
use crate::js_object::JSObject;

/// A JavaScript execution context.
pub struct JSContext {
    pub(crate) context_group: RetainReleaseWrapper<JSContextGroupRef>,
    pub(crate) inner: RetainReleaseWrapper<JSGlobalContextRef>,
}

impl fmt::Debug for JSContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JSContext").finish()
    }
}

impl Default for JSContext {
    fn default() -> Self {
        JSContext::new()
    }
}

impl JSContext {
    /// Create a new `JSContext` object.
    pub fn new() -> Self {
        let context_group = unsafe { JSContextGroupCreate() };
        let inner = unsafe { JSGlobalContextCreateInGroup(context_group, std::ptr::null_mut()) };

        Self::new_from_raw(
            context_group,
            inner,
            true
        )
    }

    pub fn from(inner: JSContextRef) -> Self {
        Self::new_from_raw(
            unsafe { JSContextGetGroup(inner) },
            unsafe { JSContextGetGlobalContext(inner) },
            false
        )
    }

    fn new_from_raw(context_group: JSContextGroupRef, inner: JSGlobalContextRef, already_retained: bool) -> Self {
        let context_group = RetainReleaseWrapper::<JSContextGroupRef>::new(
            context_group,
            already_retained,
            |x| unsafe { JSContextGroupRetain(x); }, 
            |x| unsafe { JSContextGroupRelease(x) }
        );

        let inner = 
            RetainReleaseWrapper::<JSGlobalContextRef>::new(
                inner,
                already_retained,
                |x| unsafe { JSGlobalContextRetain(x); }, 
                |x| unsafe { JSGlobalContextRelease(x); }
            );

        Self {
            context_group,
            inner,
        }
    }
}

impl JSContext {
    #[inline(always)]
    pub(crate) fn inner(&self) -> JSContextRef {
        *self.inner
    }
}

impl JSContext {
    /// Returns the context global object.
    pub fn get_global_object(&self) -> JSObject {
        JSObject::from(unsafe { JSContextGetGlobalObject(self.inner()) })
    }

    /// Evaluate the script.
    ///
    /// Returns the value the script evaluates to. If the script throws an
    /// exception, this function returns `None`. You can query the thrown
    /// exception with the `get_exception` method.
    pub fn evaluate_script(
        &mut self,
        script: &str,
        starting_line_number: i32,
    ) -> Result<JSValue, JSValue> {
        let script = JSString::from_utf8(script.to_string());
        let this_object = std::ptr::null_mut();
        let source_url = std::ptr::null_mut();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let value = unsafe {
            JSEvaluateScript(
                self.inner(),
                script.inner,
                this_object,
                source_url,
                starting_line_number,
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        Ok(JSValue::from(value))
    }
}
