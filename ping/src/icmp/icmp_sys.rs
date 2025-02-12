use std::ffi::c_void;

use crate::ipv4;

use paste::paste;

macro_rules! bind {
    ($(fn $name:ident($($arg:ident: $type:ty),*) -> $ret:ty;)*) => {
        struct Functions {
            $(pub $name: extern "stdcall" fn($($arg: $type),*) -> $ret),*
        }

        static FUNCTIONS: once_cell::sync::Lazy<Functions> = once_cell::sync::Lazy::new(|| {
            let lib = crate::loadlibrary::Library::new("IPHLPAPI.dll").unwrap();
            paste! {
                Functions {
                    $($name: unsafe { lib.get_proc(stringify!([<$name:camel>])).unwrap() }),*
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

bind! {
    fn icmp_create_file() -> Handle;
    fn icmp_send_echo(
        icmp_handle: Handle,
        destination_address: ipv4::Addr,
        request_data: *const u8,
        request_size: u16,
        request_options: Option<&IpOptionInformation>,
        reply_buffer: *mut u8,
        reply_size: u32,
        timeout: u32
    ) -> u32;
    fn icmp_close_handle(handle: Handle) -> ();
}

#[repr(C)]
#[derive(Debug)]
pub struct IpOptionInformation {
    pub ttl: u8,
    pub tos: u8,
    pub flags: u8,
    pub options_size: u8,
    pub options_data: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct IcmpEchoReply {
    pub address: ipv4::Addr,
    pub status: u32,
    pub round_trip_time: u32,
    pub data_size: u16,
    pub reserved: u16,
    pub data: *const u8,
    pub options: IpOptionInformation,
}

pub type Handle = *const c_void;
