use std::ffi::c_void;

use crate::ipv4;
use crate::loadlibrary::Library;

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

type IcmpCreateFile = extern "stdcall" fn() -> Handle;

pub fn icmp_create_file() -> Handle {
    let ip_hlp = Library::new("IPHLPAPI.dll").unwrap();
    let icmp_create_file: IcmpCreateFile = unsafe { ip_hlp.get_proc("IcmpCreateFile").unwrap() };
    icmp_create_file()
}

type IcmpSendEcho = extern "stdcall" fn(
    icmp_handle: Handle,
    destination_address: ipv4::Addr,
    request_data: *const u8,
    request_size: u16,
    request_options: Option<&IpOptionInformation>,
    reply_buffer: *mut u8,
    reply_size: u32,
    timeout: u32,
) -> u32;

pub fn icmp_send_echo(
    icmp_handle: Handle,
    destination_address: ipv4::Addr,
    request_data: *const u8,
    request_size: u16,
    request_options: Option<&IpOptionInformation>,
    reply_buffer: *mut u8,
    reply_size: u32,
    timeout: u32,
) -> u32 {
    let ip_hlp = Library::new("IPHLPAPI.dll").unwrap();
    let icmp_send_echo: IcmpSendEcho = unsafe { ip_hlp.get_proc("IcmpSendEcho").unwrap() };
    icmp_send_echo(
        icmp_handle,
        destination_address,
        request_data,
        request_size,
        request_options,
        reply_buffer,
        reply_size,
        timeout,
    )
}

type IcmpCloseHandle = extern "stdcall" fn(handle: Handle);

pub fn icmp_close_handle(handle: Handle) {
    let ip_hlp = Library::new("IPHLPAPI.dll").unwrap();
    let icmp_close_handle: IcmpCloseHandle = unsafe { ip_hlp.get_proc("IcmpCloseHandle").unwrap() };
    icmp_close_handle(handle)
}
