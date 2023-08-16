use crate::{JSContext, JSValue, JSObject, JSException};
use crate::js_protected_value::JSProtectedValue;
use rusty_jsc_sys::*;

pub struct JSPromise<T> {
    value: JSProtectedValue,
    complete: Box<dyn FnOnce(Result<T, JSException>) -> ()>,
 } 

impl<T> JSPromise<T> where T: Into<JSValue> {
    pub fn new(context: &JSContext) -> Result<Self, JSException> {
        let context = context.clone();

        let mut resolve: JSObjectRef = std::ptr::null_mut();
        let mut reject: JSObjectRef = std::ptr::null_mut();
        let mut exception: JSValueRef = std::ptr::null_mut();
        let promise: JSValue = JSValue::from(unsafe { 
            JSObjectMakeDeferredPromise(context.inner(), &mut resolve, &mut reject, &mut exception)
        });

        if !exception.is_null() {
            Err(JSException::new(&context, JSValue::from(exception)))
        } else if resolve.is_null() || reject.is_null() {
            Err(JSException::from("Missing promise resolve/reject method references"))
        } else if promise.is_null(&context) {
            Err(JSException::from("Promise could not be created, no explicit error was reported."))
        } else {
            let resolve = JSObject::from(resolve);
            let reject = JSObject::from(reject);

            unsafe { JSValueProtect(context.inner(), resolve.to_jsvalue().inner) };
            unsafe { JSValueProtect(context.inner(), reject.to_jsvalue().inner) };

            Ok(JSPromise {
                value: JSProtectedValue::new(&context, promise),
                complete: Box::new(move |result| {
                    match result {
                        Ok(value) => resolve.call(&context, None, &[value.into()]).unwrap(),
                        Err(error) => reject.call(&context, None, &[error.to_jsvalue(&context)]).unwrap()
                    };

                    unsafe { JSValueUnprotect(context.inner(), resolve.to_jsvalue().inner) };
                    unsafe { JSValueUnprotect(context.inner(), reject.to_jsvalue().inner) };        
                }),
            })
        }
    }

    pub fn done(self, result: Result<T, JSException>) {
        (self.complete)(result)
    }

    pub fn resolve(self, value: T) {
        self.done(Ok(value))
    }

    pub fn reject(self, error: JSException) {
        self.done(Err(error))
    }
}

// TODO: This is wrong becuase of the use of the cloned context internally.
// While JSContext can be cloned, it's not Sync yet and therefore should not
// be called in other threads. By making this Send, if the promise is resolved
// elsewhere then there might be a misuse of JSContext in mulitple threads at
// the same time.
unsafe impl<T> Send for JSPromise<T> {}

impl<T> From<&JSPromise<T>> for JSValue {
    fn from(promise: &JSPromise<T>) -> Self {
        promise.value.clone()
    }
}
