use std::ops::Deref;

pub trait IsRawPtr where Self: Copy {}

impl<T> IsRawPtr for *const T {}
impl<T> IsRawPtr for *mut T {}

pub struct RetainReleaseWrapper<T> where T: IsRawPtr {
    ptr: T,
    retain: fn(T),
    release: fn(T),
}

impl<T> RetainReleaseWrapper<T> where T: IsRawPtr {
    pub fn new(ptr: T, already_retained: bool, retain: fn(T), release: fn(T)) -> RetainReleaseWrapper<T> {
        if !already_retained {
            retain(ptr);
        }

        RetainReleaseWrapper { ptr, retain, release }
    }
}

impl<T> Drop for RetainReleaseWrapper<T> where T: IsRawPtr {
    fn drop(&mut self) {
        (self.release)(self.ptr);
    }
}

impl<T> Clone for RetainReleaseWrapper<T> where T: IsRawPtr {
    fn clone(&self) -> Self {
        (self.retain)(self.ptr);
        RetainReleaseWrapper {
            ptr: self.ptr,
            retain: self.retain,
            release: self.release,
        }
    }
}

impl<T> Deref for RetainReleaseWrapper<T> where T: IsRawPtr {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}
