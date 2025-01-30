mod icmp_sys;

use crate::ipv4;
use std::mem;

pub struct Request {
}

pub struct Response {
}

pub fn ping(dest: ipv4::Addr) -> Result<(), String> {
    let data = "Foo Bar Baz";
    let reply_size = mem::size_of::<icmp_sys::IcmpEchoReply>();
    let reply_buf_size = reply_size + 8 + data.len();
    let mut reply_buf = vec![0u8; reply_buf_size];

    let handle = icmp_sys::icmp_create_file();
    match icmp_sys::icmp_send_echo(
        handle,
        dest,
        data.as_ptr(),
        data.len() as u16,
        None,
        // Some(&IpOptionInformation {
        //     ttl: 128,
        //     tos: 0,
        //     flags: 0,
        //     options_data: 0,
        //     options_size: 0,
        // }),
        reply_buf.as_mut_ptr(),
        reply_buf_size as u32,
        4000,
    ) {
        0 => Err("icmp_send_echo failed".to_string()),
        _ => Ok(()),
    }
}
