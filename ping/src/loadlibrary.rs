use std::ffi::{c_void, CString};
use std::mem;
use std::os::raw::c_char;
use std::ptr::NonNull;

type HModule = NonNull<c_void>;
type FarProc = NonNull<c_void>;

extern "stdcall" {
    fn LoadLibraryA(name: *const c_char) -> Option<HModule>;
    fn GetProcAddress(module: HModule, proc_name: *const c_char) -> Option<FarProc>;
}

/// Holding an instance of `Library` means that the given DLL was loaded succesfully.
pub struct Library {
    handle: HModule,
}

impl Library {
    pub fn new(name: &str) -> Option<Self> {
        let name = CString::new(name).expect("invalid library name");
        let handle = unsafe { LoadLibraryA(name.as_ptr()) };
        handle.map(|handle| Library { handle })
    }

    pub unsafe fn get_proc<T>(&self, proc_name: &str) -> Option<T> {
        let proc_name = CString::new(proc_name).expect("invalid proc name");
        let proc = unsafe { GetProcAddress(self.handle, proc_name.as_ptr()) };
        proc.map(|proc| mem::transmute_copy(&proc))
    }
}
