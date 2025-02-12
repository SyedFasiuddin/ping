use std::fmt;
use std::ptr::NonNull;
use std::slice;

use crate::error::Error;
use crate::ethernet;
use crate::ipv4;
use crate::vls::VLS;

use custom_debug_derive::Debug;

pub fn default_nic() -> Result<NIC, Error> {
    let table = VLS::new(|ptr, size| get_ip_forward_table(ptr, size, false))?;
    let entry = table
        .entries()
        .iter()
        .find(|e| e.dest == ipv4::Addr([0, 0, 0, 0]))
        .ok_or(Error::DefaultRouteMissing)?;

    let interfaces = VLS::new(|ptr, size| get_interface_info(ptr, size))?;
    let interface = interfaces
        .adapters()
        .iter()
        .find(|a| a.index == entry.if_index)
        .ok_or(Error::DefaultInterfaceMissing)?;

    let addr_rows = VLS::new(|ptr, size| get_ip_addr_table(ptr, size, false))?;
    let address = addr_rows
        .entries()
        .iter()
        .find(|r| r.index == entry.if_index)
        .ok_or(Error::DefaultInterfaceNoIPAddr)?
        .addr;

    let mut adapter_list_head = VLS::new(|ptr, size| get_adapters_info(ptr, size))?;
    let mut current = NonNull::new(&mut *adapter_list_head);
    let mut phy_address = None;

    loop {
        if let Some(adapter) = current {
            let adapter = unsafe { adapter.as_ref() };
            if adapter.address_length == 6 && adapter.index == entry.if_index {
                phy_address = Some(adapter.address);
                break;
            }
            current = adapter.next;
        } else {
            break;
        }
    }
    let phy_address = phy_address.ok_or(Error::DefaultInterfaceNoMACAddr)?;

    let name = interface.name.to_string();
    let guid_start = name.find("{").ok_or(Error::DefaultInterfaceUnidentified)?;
    let guid = &name[guid_start..];
    Ok(NIC {
        guid: guid.to_string(),
        address,
        phy_address,
        gateway: entry.next_hop,
    })
}

#[repr(C)]
#[derive(Debug)]
pub struct IpForwardTable {
    num_entries: u32,
    entries: [IpForwardRow; 1],
}

#[repr(C)]
#[derive(Debug)]
struct IpForwardRow {
    dest: ipv4::Addr,
    mask: ipv4::Addr,
    policy: u32,
    next_hop: ipv4::Addr,
    if_index: u32,

    #[debug(skip)]
    _other_fileds: [u32; 9],
}

impl IpForwardTable {
    fn entries(&self) -> &[IpForwardRow] {
        unsafe { slice::from_raw_parts(&self.entries[0], self.num_entries as usize) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct IpInterfaceInfo {
    num_adapters: u32,
    adapter: [IpAdapterIndexMap; 1],
}

impl IpInterfaceInfo {
    fn adapters(&self) -> &[IpAdapterIndexMap] {
        unsafe { slice::from_raw_parts(&self.adapter[0], self.num_adapters as usize) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct IpAdapterIndexMap {
    index: u32,
    name: IpAdapterName,
}

pub struct IpAdapterName([u16; 128]);

impl fmt::Display for IpAdapterName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = String::from_utf16_lossy(&self.0[..]);
        write!(f, "{}", s.trim_end_matches("\0"))
    }
}

impl fmt::Debug for IpAdapterName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct IpAddrTable {
    num_entries: i32,
    entries: [IpAddrRow; 1],
}

impl IpAddrTable {
    fn entries(&self) -> &[IpAddrRow] {
        unsafe { slice::from_raw_parts(&self.entries[0], self.num_entries as usize) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct IpAddrRow {
    pub addr: ipv4::Addr,
    pub index: u32,
    pub mask: ipv4::Addr,
    pub bcast_addr: ipv4::Addr,
    pub reasm_size: u32,

    #[debug(skip)]
    unused1: u16,
    #[debug(skip)]
    unused2: u16,
}

const MAX_ADAPTER_NAME_LENGTH: usize = 256;
const MAX_ADAPTER_DESCRIPTION_LENGTH: usize = 128;

#[repr(C)]
#[derive(Debug)]
pub struct IpAdapterInfo {
    pub next: Option<NonNull<IpAdapterInfo>>,
    pub combo_index: u32,

    #[debug(skip)]
    pub adapter_name: [u8; MAX_ADAPTER_NAME_LENGTH + 4],
    #[debug(skip)]
    pub description: [u8; MAX_ADAPTER_DESCRIPTION_LENGTH + 4],

    pub address_length: u32,
    pub address: ethernet::Addr,
    pub address_rest: u16,
    pub index: u32,
    pub r#type: u32,
    // ignore rest of fields
}

#[derive(Debug)]
pub struct NIC {
    pub guid: String,
    pub gateway: ipv4::Addr,
    pub address: ipv4::Addr,
    pub phy_address: ethernet::Addr,
}

crate::bind! {
    library "IPHLPAPI.dll";

    fn get_ip_forward_table(forward_table: *mut IpForwardTable, size: *mut u32, order: bool) -> u32;
    fn get_interface_info(info: *mut IpInterfaceInfo, size: *mut u32) -> u32;

    fn get_ip_addr_table(table: *mut IpAddrTable, size: *mut u32, order: bool) -> u32;
    fn get_adapters_info(list: *mut IpAdapterInfo, size: *mut u32) -> u32;
}
