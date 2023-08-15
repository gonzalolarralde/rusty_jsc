use std::error::Error;
use std::fmt::Display;
use rusty_jsc_sys::JSValueRef;

use crate::js_context::JSContext;
use crate::js_value::JSValue;

/// A JavaScript exception, formally a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSException {
    pub value: JSValue,
    location: String,
    string_representation: Option<String>
}

impl JSException {
    #[track_caller]
    pub(crate) fn new(context: &JSContext, value: JSValue) -> Self {
        let string_representation = match value.to_string(&context) {
            Ok(str) => Some(str.to_string()),
            Err(_) => None
        };

        let location = std::panic::Location::caller().to_string();

        JSException { value, location, string_representation }
    }

    #[track_caller]
    pub fn from_string(context: &JSContext, string_representation: String) -> Self {
        let value = JSValue::string(&context, string_representation.clone());
        let location = std::panic::Location::caller().to_string();
        JSException { value, location, string_representation: Some(string_representation) }
    }
}

impl Error for JSException {}

impl Display for JSException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self.string_representation {
            Some(msg) => msg.clone(),
            None => format!("<error is not representable. JSValue: {:p}>", self.value.inner)
        };

        write!(f, "JSError {}: {}", self.location, message)
    }
}

impl Into<JSValueRef> for JSException {
    fn into(self) -> JSValueRef {
        self.value.inner
    }
}