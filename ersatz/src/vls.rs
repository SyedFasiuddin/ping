use std::marker::PhantomData;
use std::mem;
use std::ops;
use std::ptr;

use crate::error::Error;

const ERROR_INSUFFICIENT_MEMORY: u32 = 122;
const ERROR_BUFFER_OVERFLOW: u32 = 111;

pub struct VLS<T> {
    v: Vec<u8>,
    _phantom: PhantomData<T>,
}

impl<T> ops::Deref for VLS<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self.v.as_ptr()) }
    }
}

impl<T> VLS<T> {
    pub fn new<F>(f: F) -> Result<Self, Error>
    where
        F: Fn(*mut T, *mut u32) -> u32,
    {
        let mut size = 0;
        match f(ptr::null_mut(), &mut size) {
            ERROR_INSUFFICIENT_MEMORY | ERROR_BUFFER_OVERFLOW => (),
            ret => return Err(Error::Win32(ret)),
        }
        let mut v = vec![0u8; size as usize];
        match f(unsafe { mem::transmute(v.as_mut_ptr()) }, &mut size) {
            0 => (),
            ret => return Err(Error::Win32(ret)),
        }

        Ok(Self {
            v,
            _phantom: PhantomData::default(),
        })
    }
}

impl<T> std::ops::DerefMut for VLS<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { mem::transmute(self.v.as_ptr()) }
    }
}
