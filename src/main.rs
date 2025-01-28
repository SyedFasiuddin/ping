use std::ffi::c_void;
use std::fmt;
use std::mem;
use pretty_hex::PrettyHex;

type HModule = *const c_void;
type FarProc = *const c_void;
type Handle = *const c_void;

type IcmpCreateFile = extern "stdcall" fn() -> Handle;
type IcmpSendEcho = extern "stdcall" fn(
    icmp_handle: Handle,
    destination_address: IPAddr,
    request_data: *const u8,
    request_size: u16,
    request_options: Option<&IpOptionInformation>,
    reply_buffer: *mut u8,
    reply_size: u32,
    timeout: u32,
) -> u32;

extern "stdcall" {
    fn LoadLibraryA(name: *const u8) -> HModule;
    fn GetProcAddress(module: HModule, proc_name: *const u8) -> FarProc;
}

#[repr(C)]
struct IPAddr([u8; 4]);

impl fmt::Debug for IPAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d] = self.0;
        write!(f, "{}.{}.{}.{}", a, b, c, d)
    }
}

#[repr(C)]
struct IpOptionInformation {
    ttl: u8,
    tos: u8,
    flags: u8,
    options_size: u8,
    options_data: u32,
}

fn main() {
    let ip_hlp_api = unsafe { LoadLibraryA("IPHLPAPI.dll\0".as_ptr()) };
    let icmp_create_file: IcmpCreateFile =
        unsafe { mem::transmute(GetProcAddress(ip_hlp_api, "IcmpCreateFile\0".as_ptr())) };
    let icmp_send_echo: IcmpSendEcho =
        unsafe { mem::transmute(GetProcAddress(ip_hlp_api, "IcmpSendEcho\0".as_ptr())) };

    let data = "Foo Bar Baz";
    let mut reply_buf = vec![0u8; 128];

    let handle = icmp_create_file();
    let ret = icmp_send_echo(
        handle,
        IPAddr([8, 8, 8, 8]),
        data.as_ptr(),
        data.len() as u16,
        Some(&IpOptionInformation {
            ttl: 128,
            tos: 0,
            flags: 0,
            options_data: 0,
            options_size: 0,
        }),
        reply_buf.as_mut_ptr(),
        reply_buf.len() as u32,
        4000,
    );

    println!("ret: {}", ret);
    println!("{:?}", reply_buf.hex_dump());
}
