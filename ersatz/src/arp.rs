use crate::parse;
use crate::{ethernet, ipv4};
use cookie_factory as cf;
use derive_try_from_primitive::TryFromPrimitive;

use std::io;

#[derive(Debug, TryFromPrimitive, Clone, Copy)]
#[repr(u16)]
pub enum Operation {
    Request = 1,
    Reply = 2,
}

impl Operation {
    pub fn parse(i: parse::Input) -> parse::Result<Option<Self>> {
        use nom::{combinator::map, number::complete::be_u16};
        map(be_u16, |x| Self::try_from(x).ok())(i)
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::bytes::be_u16;
        be_u16(*self as u16)
    }
}

#[derive(Debug, TryFromPrimitive, Clone, Copy)]
#[repr(u16)]
pub enum HardwareType {
    Ethernet = 1,
}

impl HardwareType {
    pub fn parse(i: parse::Input) -> parse::Result<Option<Self>> {
        use nom::{combinator::map, number::complete::be_u16};
        map(be_u16, |x| Self::try_from(x).ok())(i)
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::bytes::be_u16;
        be_u16(*self as u16)
    }
}

#[derive(Debug)]
pub struct Packet {
    pub operation: Operation,
    pub sender_hw_addr: ethernet::Addr,
    pub sender_ip_addr: ipv4::Addr,
    pub target_hw_addr: ethernet::Addr,
    pub target_ip_addr: ipv4::Addr,
}

impl Packet {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        use nom::{number::complete::be_u8, sequence::tuple};

        let (i, (_htype, _ptype, _hlen, _plen)) = tuple((
            HardwareType::parse,
            ethernet::EtherType::parse,
            be_u8,
            be_u8,
        ))(i)?;

        let (i, operation) = Operation::parse(i)?;
        let operation = match operation {
            Some(operation) => operation,
            _ => panic!(),
        };

        let (i, (sender_hw_addr, sender_ip_addr)) =
            tuple((ethernet::Addr::parse, ipv4::Addr::parse))(i)?;

        let (i, (target_hw_addr, target_ip_addr)) =
            tuple((ethernet::Addr::parse, ipv4::Addr::parse))(i)?;

        let res = Self {
            operation,
            sender_hw_addr,
            sender_ip_addr,
            target_hw_addr,
            target_ip_addr,
        };
        Ok((i, res))
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{bytes::be_u8, sequence::tuple};

        let htype = HardwareType::Ethernet.serialize();
        let ptype = ethernet::EtherType::IPv4.serialize();
        let hlen = be_u8(6);
        let plen = be_u8(4);
        tuple((
            htype,
            ptype,
            hlen,
            plen,
            self.operation.serialize(),
            self.sender_hw_addr.serialize(),
            self.sender_ip_addr.serialize(),
            self.target_hw_addr.serialize(),
            self.target_ip_addr.serialize(),
        ))
    }

    pub fn request(nic: &crate::netinfo::NIC, target_ip_addr: ipv4::Addr) -> Self {
        Self {
            operation: Operation::Request,
            sender_ip_addr: nic.address,
            sender_hw_addr: nic.phy_address,
            target_ip_addr,
            target_hw_addr: ethernet::Addr::zero(),
        }
    }

    pub fn as_ethernet_payload(self) -> ethernet::Payload {
        ethernet::Payload::ARP(self)
    }
}
