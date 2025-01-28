type HModule = *const std::ffi::c_void;
type FarProc = *const std::ffi::c_void;

extern "stdcall" {
    fn LoadLibraryA(name: *const u8) -> HModule;
    fn GetProcAddress(module: HModule, procName: *const u8) -> FarProc;
}

type MessageBoxA = extern "stdcall" fn(*const std::ffi::c_void, *const u8, *const u8, u32);

fn main() {
    let h = unsafe { LoadLibraryA("USER32.dll\0".as_ptr()) };
    let MessageBoxA: MessageBoxA =
        unsafe { std::mem::transmute(GetProcAddress(h, "MessageBoxA\0".as_ptr())) };

    MessageBoxA(
        std::ptr::null(),
        "Hello from Rust\0".as_ptr(),
        std::ptr::null(),
        0,
    );
}
