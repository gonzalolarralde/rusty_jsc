use crate::internal::JSString;
use rusty_jsc_sys::JSObjectCallAsFunctionCallback;
use rusty_jsc_sys::*;

use crate::js_context::JSContext;
use crate::js_value::JSValue;

/// A JavaScript object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSObject {
    inner: JSObjectRef,
}

unsafe impl Send for JSObject {}
unsafe impl Sync for JSObject {}

impl Drop for JSObject {
    fn drop(&mut self) {
        // TODO
    }
}

impl JSObject {
    /// Wraps a `JSObject` from a `JSObjectRef`.
    fn from(inner: JSObjectRef) -> Self {
        Self { inner }
    }

    pub fn new(context: &JSContext) -> Self {
        let null = std::ptr::null_mut();
        let o_ref = unsafe { JSObjectMake(context.inner, null, null as _) };
        Self::from(o_ref)
    }

    /// Create a new Array Object with the given arguments
    pub fn new_array(context: &JSContext, args: &[JSValue]) -> Result<Self, JSValue> {
        let args_refs = args.iter().map(|arg| arg.inner).collect::<Vec<_>>();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let o_ref = unsafe {
            JSObjectMakeArray(
                context.inner,
                args.len() as _,
                args_refs.as_slice().as_ptr(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        Ok(Self::from(o_ref))
    }

    pub fn new_function_with_callback(
        context: &JSContext,
        name: impl Into<JSString>,
        callback: JSObjectCallAsFunctionCallback,
    ) -> Self {
        let name = name.into();
        let o_ref =
            unsafe { JSObjectMakeFunctionWithCallback(context.inner, name.inner, callback) };
        Self::from(o_ref)
    }

    /// Calls the object constructor
    pub fn construct(&self, context: &JSContext, args: &[JSValue]) -> Result<Self, JSValue> {
        let args_refs = args.iter().map(|arg| arg.inner).collect::<Vec<_>>();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let result = unsafe {
            JSObjectCallAsConstructor(
                context.inner,
                self.inner,
                args.len() as _,
                args_refs.as_slice().as_ptr(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        if result.is_null() {
            return Err(JSValue::string(
                context,
                format!(
                    "Can't call constructor for {:?}: not a valid constructor",
                    self.to_jsvalue().to_string(context)
                ),
            ));
        }
        Ok(Self::from(result))
    }

    /// Call the object as if it a function
    pub fn call(
        &self,
        context: &JSContext,
        this: Option<&JSObject>,
        args: &[JSValue],
    ) -> Result<JSValue, JSValue> {
        let args_refs = args.iter().map(|arg| arg.inner).collect::<Vec<_>>();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let result = unsafe {
            JSObjectCallAsFunction(
                context.inner,
                self.inner,
                this.map(|t| t.inner)
                    .unwrap_or_else(|| std::ptr::null_mut()),
                args.len() as _,
                args_refs.as_slice().as_ptr(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        if result.is_null() {
            return Err(JSValue::string(
                context,
                format!(
                    "Can't call the object {:?}: not a valid function",
                    self.to_jsvalue().to_string(context)
                ),
            ));
        }
        Ok(JSValue::from(result))
    }

    /// Calls the object constructor
    pub fn to_jsvalue(&self) -> JSValue {
        JSValue::from(self.inner)
    }

    pub fn create_typed_array_with_bytes(
        context: &JSContext,
        bytes: &mut [u8],
    ) -> Result<Self, JSValue> {
        let deallocator_ctx = std::ptr::null_mut();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let result = unsafe {
            JSObjectMakeTypedArrayWithBytesNoCopy(
                context.inner,
                JSTypedArrayType_kJSTypedArrayTypeUint8Array,
                bytes.as_mut_ptr() as _,
                bytes.len() as _,
                None,
                deallocator_ctx,
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        if result.is_null() {
            return Err(JSValue::string(
                context,
                "Can't create a type array".to_string(),
            ));
        }
        Ok(Self::from(result))
    }

    pub fn create_typed_array_from_buffer(
        context: &JSContext,
        buffer: JSObject,
    ) -> Result<Self, JSValue> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let result = unsafe {
            JSObjectMakeTypedArrayWithArrayBuffer(
                context.inner,
                JSTypedArrayType_kJSTypedArrayTypeUint8Array,
                buffer.inner,
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        if result.is_null() {
            return Err(JSValue::string(
                context,
                "Can't create a typed array from the provided buffer".to_string(),
            ));
        }
        Ok(Self::from(result))
    }

    pub fn get_typed_array_buffer(&self, context: &JSContext) -> Result<&mut [u8], JSValue> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let arr_ptr =
            unsafe { JSObjectGetTypedArrayBytesPtr(context.inner, self.inner, &mut exception) };
        let arr_len =
            unsafe { JSObjectGetTypedArrayLength(context.inner, self.inner, &mut exception) };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        let slice = unsafe { std::slice::from_raw_parts_mut(arr_ptr as _, arr_len as usize) };
        Ok(slice)
    }

    /// Gets the property of an object.
    pub fn get_property(&self, context: &JSContext, property_name: impl Into<JSString>) -> JSValue {
        let property_name = property_name.into();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let jsvalue_ref = unsafe {
            JSObjectGetProperty(
                context.inner,
                self.inner,
                property_name.inner,
                &mut exception,
            )
        };
        JSValue::from(jsvalue_ref)
    }

    /// Gets the property of an object at a given index
    pub fn get_property_at_index(
        &self,
        context: &JSContext,
        property_index: u32,
    ) -> Result<JSValue, JSValue> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let property = unsafe {
            JSObjectGetPropertyAtIndex(context.inner, self.inner, property_index, &mut exception)
        };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        Ok(JSValue::from(property))
    }

    pub fn get_property_names(&mut self, context: &JSContext) -> Vec<String> {
        let property_name_array = unsafe { JSObjectCopyPropertyNames(context.inner, self.inner) };
        let num_properties = unsafe { JSPropertyNameArrayGetCount(property_name_array) };
        (0..num_properties)
            .map(|property_index| {
                JSString::from(unsafe {
                    JSPropertyNameArrayGetNameAtIndex(property_name_array, property_index)
                })
                .to_string()
            })
            .collect::<Vec<_>>()
    }

    // Get the object as an array buffer
    pub fn get_array_buffer(&mut self, context: &JSContext) -> Result<&mut [u8], JSValue> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let arr_ptr =
            unsafe { JSObjectGetArrayBufferBytesPtr(context.inner, self.inner, &mut exception) };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        let arr_len =
            unsafe { JSObjectGetArrayBufferByteLength(context.inner, self.inner, &mut exception) };
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        let slice = unsafe { std::slice::from_raw_parts_mut(arr_ptr as _, arr_len as usize) };
        Ok(slice)
    }

    /// Sets the property of an object.
    pub fn set_property(
        &self,
        context: &JSContext,
        property_name: impl Into<JSString>,
        value: JSValue,
    ) -> Result<(), JSValue> {
        let property_name = property_name.into();
        let attributes = 0; // TODO
        let mut exception: JSValueRef = std::ptr::null_mut();
        unsafe {
            JSObjectSetProperty(
                context.inner,
                self.inner,
                property_name.inner,
                value.inner,
                attributes,
                &mut exception,
            )
        }
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        Ok(())
    }

    /// Sets the property of an object at a given index
    pub fn set_property_at_index(
        &self,
        context: &JSContext,
        index: u32,
        value: JSValue,
    ) -> Result<(), JSValue> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        unsafe {
            JSObjectSetPropertyAtIndex(
                context.inner,
                self.inner,
                index,
                value.inner,
                &mut exception,
            )
        }
        if !exception.is_null() {
            return Err(JSValue::from(exception));
        }
        Ok(())
    }
}

impl From<JSObjectRef> for JSObject {
    fn from(obj: JSObjectRef) -> Self {
        JSObject::from(obj)
    }
}

impl From<JSValueRef> for JSValue {
    fn from(val: JSValueRef) -> Self {
        JSValue::from(val)
    }
}

impl From<JSValue> for JSValueRef {
    fn from(val: JSValue) -> Self {
        val.inner
    }
}

impl From<JSObject> for JSObjectRef {
    fn from(val: JSObject) -> JSObjectRef {
        val.inner
    }
}
