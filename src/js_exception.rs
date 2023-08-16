use std::error::Error;
use std::fmt::Display;

use crate::js_context::JSContext;
use crate::js_value::JSValue;

#[derive(Debug, Clone, PartialEq, Eq)]
enum JSExceptionBody {
    JSValue(JSValue),
    JSValueWithRepresentation(JSValue, String),
    String(String),
}

/// A JavaScript exception, formally a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSException {
    body: JSExceptionBody,
    location: String,
}

impl JSException {
    #[track_caller]
    pub fn new(context: &JSContext, value: JSValue) -> Self {
        let location = std::panic::Location::caller().to_string();

        let string_representation = match value.to_string(&context) {
            Ok(str) => Some(str.to_string()),
            Err(_) => None
        };

        let body = if let Some(string_representation) = string_representation {
            JSExceptionBody::JSValueWithRepresentation(value, string_representation.clone())
        } else {
            JSExceptionBody::JSValue(value)
        };

        JSException { body: body.into(), location }
    }

    pub fn to_jsvalue(&self, context: &JSContext) -> JSValue {
        match &self.body {
            JSExceptionBody::JSValue(value) => value.clone(),
            JSExceptionBody::JSValueWithRepresentation(value, _) => value.clone(),
            JSExceptionBody::String(string) => JSValue::string(&context, string.clone()),
        }
    }
}

impl Error for JSException {}

impl Display for JSException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self.body {
            JSExceptionBody::JSValue(value) => format!("<error is not representable. JSValueRef={:p}>", value.inner),
            JSExceptionBody::JSValueWithRepresentation(_, msg) => msg.clone(),
            JSExceptionBody::String(msg) => msg.clone(),
        };

        write!(f, "JSException {}: {}", self.location, message)
    }
}

impl From<String> for JSException {
    #[track_caller]
    fn from(string: String) -> Self {
        let location = std::panic::Location::caller().to_string();
        JSException { body: JSExceptionBody::String(string).into(), location }
    }
}

impl From<&str> for JSException {
    #[track_caller]
    fn from(string: &str) -> Self {
        let location = std::panic::Location::caller().to_string();
        JSException { body: JSExceptionBody::String(string.to_string()).into(), location }
    }
}