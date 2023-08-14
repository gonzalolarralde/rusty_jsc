use crate::internal::JSString;
use rusty_jsc_sys::JSObjectCallAsFunctionCallback;
use rusty_jsc_sys::*;

use crate::js_context::JSContext;
use crate::js_object::JSObject;

/// A JavaScript value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSValue {
    pub(crate) inner: JSValueRef,
}

impl Drop for JSValue {
    fn drop(&mut self) {
        // TODO
    }
}

impl JSValue {
    /// Wraps a `JSValue` from a `JSValueRef`.
    pub(crate) fn from(inner: JSValueRef) -> Self {
        Self { inner }
    }

    /// Creates an `undefined` value.
    pub fn undefined(context: &JSContext) -> JSValue {
        JSValue::from(unsafe { JSValueMakeUndefined(context.inner) })
    }

    /// Creates a `null` value.
    pub fn null(context: &JSContext) -> JSValue {
        JSValue::from(unsafe { JSValueMakeNull(context.inner) })
    }

    /// Creates a `boolean` value.
    pub fn boolean(context: &JSContext, value: bool) -> JSValue {
        JSValue::from(unsafe { JSValueMakeBoolean(context.inner, value) })
    }

    /// Creates a `number` value.
    pub fn number(context: &JSContext, value: f64) -> JSValue {
        JSValue::from(unsafe { JSValueMakeNumber(context.inner, value) })
    }

    /// Creates a `string` value.
    pub fn string(context: &JSContext, value: impl Into<JSString>) -> JSValue {
        let value = value.into();
        JSValue::from(unsafe { JSValueMakeString(context.inner, value.inner) })
    }

    pub fn callback(context: &JSContext, callback: JSObjectCallAsFunctionCallback) -> JSValue {
        let name = JSString::from_utf8("".to_string());
        let func = unsafe { JSObjectMakeFunctionWithCallback(context.inner, name.inner, callback) };
        JSValue::from(func)
    }

    /// Checks if this value is `undefined`.
    pub fn is_undefined(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsUndefined(context.inner, self.inner) }
    }

    /// Checks if this value is `null`.
    pub fn is_null(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsNull(context.inner, self.inner) }
    }

    /// Checks if this value is `boolean`.
    pub fn is_boolean(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsBoolean(context.inner, self.inner) }
    }

    /// Checks if this value is `Array`.
    pub fn is_array(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsArray(context.inner, self.inner) }
    }

    /// Checks if this value is `number`.
    pub fn is_number(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsNumber(context.inner, self.inner) }
    }

    /// Checks if this value is `string`.
    pub fn is_string(&self, context: &JSContext) -> bool {
        unsafe { JSValueIsString(context.inner, self.inner) }
    }

    /// Gets this value as a `bool`.
    pub fn to_bool(&self, context: &JSContext) -> bool {
        unsafe { JSValueToBoolean(context.inner, self.inner) }
    }

    /// Formats this value as a `String`.
    pub fn to_string(&self, context: &JSContext) -> Result<JSString, JSValue> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let string = unsafe { JSValueToStringCopy(context.inner, self.inner, &mut exception) };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        Ok(JSString::from(string))
    }

    // Tries to convert the value to a number
    pub fn to_number(&self, context: &JSContext) -> Result<f64, JSValue> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let num = unsafe { JSValueToNumber(context.inner, self.inner, &mut exception) };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        Ok(num)
    }

    // Tries to convert the value to an object
    pub fn to_object(&self, context: &JSContext) -> Result<JSObject, JSValue> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let object_ref = unsafe { JSValueToObject(context.inner, self.inner, &mut exception) };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        let obj = JSObject::from(object_ref);
        Ok(obj)
    }
}

unsafe impl Send for JSValue {}
unsafe impl Sync for JSValue {}
