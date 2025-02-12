use crate::arp;
use crate::ipv4;
use crate::parse;

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::TryFromPrimitive;
use nom::{
    bytes::complete::take, combinator::map, number::complete::be_u16, sequence::tuple, Parser,
};

use std::fmt;
use std::io;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Addr([u8; 6]);

impl fmt::Display for Addr {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        let [a, b, c, d, e, f] = self.0;
        write!(
            w,
            "{:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}",
            a, b, c, d, e, f
        )
    }
}

impl fmt::Debug for Addr {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, w)
    }
}

impl Addr {
    /// This will panic if the given slice doesn't have atleast 6 bytes
    pub fn new(slice: &[u8]) -> Self {
        let mut res = Self([0u8; 6]);
        res.0.copy_from_slice(&slice[..6]);
        res
    }

    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        (map(take(6_usize), Self::new)).parse(i)
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::combinator::slice;
        slice(&self.0)
    }

    pub fn zero() -> Self {
        Self([0, 0, 0, 0, 0, 0])
    }

    pub fn broadcast() -> Self {
        Self([0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF])
    }
}

#[derive(Debug)]
pub struct Frame {
    pub dst: Addr,
    pub src: Addr,
    pub ether_type: Option<EtherType>,
    pub payload: Payload,
}

#[derive(Debug)]
pub enum Payload {
    IPv4(ipv4::Packet),
    ARP(arp::Packet),
    Unknown,
}

impl Payload {
    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::sequence::tuple;
        move |out| match self {
            Self::ARP(ref packet) => tuple((EtherType::ARP.serialize(), packet.serialize()))(out),
            Self::IPv4(ref packet) => tuple((EtherType::IPv4.serialize(), packet.serialize()))(out),
            Self::Unknown => unimplemented!(),
        }
    }

    pub fn as_frame(self, nic: &crate::netinfo::NIC, dst: Addr) -> Frame {
        Frame {
            src: nic.phy_address,
            dst,
            ether_type: None,
            payload: self,
        }
    }

    pub fn as_broadcast_frame(self, nic: &crate::netinfo::NIC) -> Frame {
        self.as_frame(nic, Addr::broadcast())
    }
}

#[derive(Debug, TryFromPrimitive, Clone, Copy)]
#[repr(u16)]
pub enum EtherType {
    IPv4 = 0x0800,
    ARP = 0x0806,
}

impl EtherType {
    pub fn parse(i: parse::Input) -> parse::Result<Option<Self>> {
        map(be_u16, |x| Self::try_from(x).ok()).parse(i)
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::bytes::be_u16;
        // note: I had to derive `Clone` and `Copy` to make this work
        // those weren't originally derived for `ethernet::EtherType`
        be_u16(*self as u16)
    }
}

impl Frame {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, (dst, src)) = tuple((Addr::parse, Addr::parse)).parse(i)?;
        let (i, ether_type) = EtherType::parse(i)?;

        let (i, payload) = match ether_type {
            Some(EtherType::IPv4) => map(ipv4::Packet::parse, Payload::IPv4).parse(i)?,
            Some(EtherType::ARP) => map(arp::Packet::parse, Payload::ARP).parse(i)?,
            None => (i, Payload::Unknown),
        };

        let res = Self {
            dst,
            src,
            ether_type,
            payload,
        };
        Ok((i, res))
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::sequence::tuple;
        tuple((
            self.dst.serialize(),
            self.src.serialize(),
            self.payload.serialize(),
        ))
    }

    pub fn send(&self, iface: &dyn rawsock::traits::DynamicInterface) {
        let serialized = cf::gen_simple(self.serialize(), Vec::new()).unwrap();
        iface.send(&serialized).unwrap();
    }
}
