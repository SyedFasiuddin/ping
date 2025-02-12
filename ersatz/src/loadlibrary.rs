use std::ffi::{c_void, CString};
use std::mem;
use std::os::raw::c_char;
use std::ptr::NonNull;
use thiserror::Error;

type HModule = NonNull<c_void>;
type FarProc = NonNull<c_void>;

extern "stdcall" {
    fn LoadLibraryA(name: *const c_char) -> Option<HModule>;
    fn GetProcAddress(module: HModule, proc_name: *const c_char) -> Option<FarProc>;
}

#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("invalid library name: {0}")]
    InvalidName(#[from] std::ffi::NulError),

    #[error("could not open library {0:?}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum ProcError {
    #[error("invalid proc name: {0}")]
    InvalidName(#[from] std::ffi::NulError),

    #[error("could not find proc {proc:?} in {library:?}")]
    NotFound { proc: String, library: String },
}

/// Holding an instance of `Library` means that the given DLL was loaded succesfully.
pub struct Library {
    name: String,
    handle: HModule,
}

impl Library {
    pub fn new(name: &str) -> Result<Self, LibraryError> {
        let c_name = CString::new(name)?;
        match unsafe { LoadLibraryA(c_name.as_ptr()) } {
            Some(handle) => Ok(Self {
                name: name.to_string(),
                handle,
            }),
            None => Err(LibraryError::NotFound(name.to_string()))?,
        }
    }

    pub fn get_proc<T>(&self, proc_name: &str) -> Result<T, ProcError> {
        let c_proc_name = CString::new(proc_name)?;

        match unsafe { GetProcAddress(self.handle, c_proc_name.as_ptr()) } {
            Some(proc) => Ok(unsafe { mem::transmute_copy(&proc) }),
            None => Err(ProcError::NotFound {
                library: self.name.clone(),
                proc: proc_name.to_string(),
            })?,
        }
    }
}

#[macro_export]
macro_rules! bind {
    (library $lib:expr; $(fn $name:ident($($arg:ident: $type:ty),*) -> $ret:ty;)*) => {
        struct Functions {
            $(pub $name: extern "stdcall" fn($($arg: $type),*) -> $ret),*
        }

        static FUNCTIONS: once_cell::sync::Lazy<Functions> = once_cell::sync::Lazy::new(|| {
            let lib = crate::loadlibrary::Library::new($lib).unwrap();
            paste::paste! {
                Functions {
                    $($name: { lib.get_proc(stringify!([<$name:camel>])).unwrap() }),*
                }
            }
        });

        $(
            #[inline(always)]
            pub fn $name($($arg: $type),*) -> $ret {
                (FUNCTIONS.$name)($($arg),*)
            }
        )*
    };
}
