use std::ffi::c_void;

use crate::ipv4;
use crate::loadlibrary::Library;

use once_cell::sync::Lazy;

pub static FUNCTIONS: Lazy<Functions> = Lazy::new(|| Functions::get());

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

pub struct Functions {
    pub icmp_create_file: extern "stdcall" fn() -> Handle,
    pub icmp_send_echo: extern "stdcall" fn(
        icmp_handle: Handle,
        destination_address: ipv4::Addr,
        request_data: *const u8,
        request_size: u16,
        request_options: Option<&IpOptionInformation>,
        reply_buffer: *mut u8,
        reply_size: u32,
        timeout: u32,
    ) -> u32,
    pub icmp_close_handle: extern "stdcall" fn(handle: Handle),
}

impl Functions {
    fn get() -> Self {
        let ip_hlp = Library::new("IPHLPAPI.dll").unwrap();
        Self {
            icmp_create_file: unsafe { ip_hlp.get_proc("IcmpCreateFile").unwrap() },
            icmp_send_echo: unsafe { ip_hlp.get_proc("IcmpSendEcho").unwrap() },
            icmp_close_handle: unsafe { ip_hlp.get_proc("IcmpCloseHandle").unwrap() },
        }
    }
}
