use crate::internal::JSString;
use rusty_jsc_sys::JSObjectCallAsFunctionCallback;
use rusty_jsc_sys::*;
use bytes::Bytes;
use std::{ptr, os::raw::c_void};

use crate::js_context::JSContext;
use crate::js_value::JSValue;
use crate::JSException;

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
        let o_ref = unsafe { JSObjectMake(context.inner(), null, null as _) };
        Self::from(o_ref)
    }

    /// Create a new Array Object with the given arguments
    pub fn new_array(context: &JSContext, args: &[JSValue]) -> Result<Self, JSException> {
        let args_refs = args.iter().map(|arg| arg.inner).collect::<Vec<_>>();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let o_ref = unsafe {
            JSObjectMakeArray(
                context.inner(),
                args.len() as _,
                args_refs.as_slice().as_ptr(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
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
            unsafe { JSObjectMakeFunctionWithCallback(context.inner(), name.inner, callback) };
        Self::from(o_ref)
    }

    /// Calls the object constructor
    pub fn construct(&self, context: &JSContext, args: &[JSValue]) -> Result<Self, JSException> {
        let args_refs = args.iter().map(|arg| arg.inner).collect::<Vec<_>>();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let result = unsafe {
            JSObjectCallAsConstructor(
                context.inner(),
                self.inner,
                args.len() as _,
                args_refs.as_slice().as_ptr(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
        }
        if result.is_null() {
            return Err(JSException::from(
                format!(
                    "Can't call constructor for {:?}: not a valid constructor",
                    self.to_jsvalue().to_string(context)
                )
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
    ) -> Result<JSValue, JSException> {
        let args_refs = args.iter().map(|arg| arg.inner).collect::<Vec<_>>();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let result = unsafe {
            JSObjectCallAsFunction(
                context.inner(),
                self.inner,
                this.map(|t| t.inner)
                    .unwrap_or_else(|| std::ptr::null_mut()),
                args.len() as _,
                args_refs.as_slice().as_ptr(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
        }
        if result.is_null() {
            return Err(JSException::from(
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
        bytes: Bytes,
    ) -> Result<Self, JSException> {
        let len = bytes.len();
        let data_ptr = bytes.as_ptr() as *mut u8;
    
        extern "C" fn deallocate_bytes(_ptr: *mut c_void, context: *mut c_void) {
            unsafe {
                // Convert the raw pointer back into Box<Bytes> and then drop it.
                let _: Box<Bytes> = Box::from_raw(context as *mut Bytes);
                // The Box goes out of scope here and deallocates the Bytes.
            }
        }
    
        // Leak the Bytes to ensure it doesn't get prematurely deallocated.
        // We will clean up in the deallocate_bytes function.
        let leaked_bytes = Box::leak(Box::new(bytes));
    
        let mut exception: JSValueRef = std::ptr::null_mut();
        let result = unsafe {
            JSObjectMakeTypedArrayWithBytesNoCopy(
                context.inner(),
                JSTypedArrayType_kJSTypedArrayTypeUint8Array,
                data_ptr as _,
                len as _,
                Some(deallocate_bytes),
                leaked_bytes as *const _ as *mut _,
                &mut exception,
            )
        };

        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
        }
        if result.is_null() {
            // An exception would cause the deallocator to be called, but there's no explicit reference
            // for when NULL is returned but no exception was thrown. Might not even be possible for
            // that to happen. TODO: Investigate under what circumstances, if any, this scenario could
            // happen and figure out what is the right approach to ensure a correct memory management.
            deallocate_bytes(ptr::null_mut(), leaked_bytes as *const _ as *mut _);
            return Err(JSException::from("Can't create a typed array"));
        }

        let object = Self::from(result);

        // There's no need to explicitly call the deallocator in case of a failure here because the
        // array creation succeeded, so what should happen in case of a failure here is that the
        // garbage collector should take away the failed-to-be-created reference and _that_ should
        // call the deallocator.
        context.get_global_object()
            .get_property(&context, "Object").to_object(&context)?
            .get_property(&context, "freeze").to_object(&context)?
            .call(&context, None, &[object.to_jsvalue()])?;

        Ok(object)
    }

    pub fn create_typed_array_from_buffer(
        context: &JSContext,
        buffer: JSObject,
    ) -> Result<Self, JSException> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let result = unsafe {
            JSObjectMakeTypedArrayWithArrayBuffer(
                context.inner(),
                JSTypedArrayType_kJSTypedArrayTypeUint8Array,
                buffer.inner,
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
        }
        if result.is_null() {
            return Err(JSException::from("Can't create a typed array from the provided buffer"));
        }
        Ok(Self::from(result))
    }

    pub fn get_typed_array_buffer(&self, context: &JSContext) -> Result<&mut [u8], JSException> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let arr_ptr =
            unsafe { JSObjectGetTypedArrayBytesPtr(context.inner(), self.inner, &mut exception) };
        let arr_len =
            unsafe { JSObjectGetTypedArrayLength(context.inner(), self.inner, &mut exception) };
        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
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
                context.inner(),
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
    ) -> Result<JSValue, JSException> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let property = unsafe {
            JSObjectGetPropertyAtIndex(context.inner(), self.inner, property_index, &mut exception)
        };
        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
        }
        Ok(JSValue::from(property))
    }

    pub fn get_property_names(&mut self, context: &JSContext) -> Vec<String> {
        let property_name_array = unsafe { JSObjectCopyPropertyNames(context.inner(), self.inner) };
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
    pub fn get_array_buffer(&mut self, context: &JSContext) -> Result<&mut [u8], JSException> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        let arr_ptr =
            unsafe { JSObjectGetArrayBufferBytesPtr(context.inner(), self.inner, &mut exception) };
        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
        }
        let arr_len =
            unsafe { JSObjectGetArrayBufferByteLength(context.inner(), self.inner, &mut exception) };
        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
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
    ) -> Result<(), JSException> {
        let property_name = property_name.into();
        let attributes = 0; // TODO
        let mut exception: JSValueRef = std::ptr::null_mut();
        unsafe {
            JSObjectSetProperty(
                context.inner(),
                self.inner,
                property_name.inner,
                value.inner,
                attributes,
                &mut exception,
            )
        }
        if !exception.is_null() {
            return Err(JSException::new(context, JSValue::from(exception)));
        }
        Ok(())
    }

    /// Sets the property of an object at a given index
    pub fn set_property_at_index(
        &self,
        context: &JSContext,
        index: u32,
        value: JSValue,
    ) -> Result<(), JSException> {
        let mut exception: JSValueRef = std::ptr::null_mut();
        unsafe {
            JSObjectSetPropertyAtIndex(
                context.inner(),
                self.inner,
                index,
                value.inner,
                &mut exception,
            )
        }
        if !exception.is_null() {
            return Err(JSException::new(&context, JSValue::from(exception)));
        }
        Ok(())
    }
}

impl From<JSObjectRef> for JSObject {
    fn from(obj: JSObjectRef) -> Self {
        JSObject::from(obj)
    }
}

impl From<JSObject> for JSObjectRef {
    fn from(val: JSObject) -> JSObjectRef {
        val.inner
    }
}

impl From<JSObject> for JSValue {
    fn from(value: JSObject) -> Self {
        JSValue { inner: value.inner }
    }
}
