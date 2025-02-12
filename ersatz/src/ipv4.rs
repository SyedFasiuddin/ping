use crate::icmp;
use crate::parse::{self, BitParsable};

use std::io;
use std::num::ParseIntError;

use cookie_factory as cf;
use custom_debug_derive::Debug;
use derive_try_from_primitive::TryFromPrimitive;
use nom::{
    bits::bits,
    bytes::complete::take,
    combinator::map,
    number::complete::{be_u16, be_u8},
    sequence::tuple,
    Parser,
};
use ux::*;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Addr(pub [u8; 4]);

impl Addr {
    pub fn zero() -> Self {
        Self([0, 0, 0, 0])
    }

    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, slice) = take(4_usize).parse(i)?;
        let mut res = Self([0, 0, 0, 0]);
        res.0.copy_from_slice(slice);
        Ok((i, res))
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::combinator::slice;
        slice(&self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseAddrError {
    #[error("not enough parts")]
    NotEnoughParts,
    #[error("too many parts")]
    TooManyParts,
    #[error("one of the components is not an int {0:?}")]
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for ParseAddrError {
    fn from(e: ParseIntError) -> Self {
        ParseAddrError::ParseIntError(e)
    }
}

// impl std::error::Error for ParseAddrError {}

impl std::str::FromStr for Addr {
    type Err = ParseAddrError;

    fn from_str(s: &str) -> Result<Self, ParseAddrError> {
        let mut tokens = s.split(".");

        let mut res = Self([0, 0, 0, 0]);
        for part in res.0.iter_mut() {
            *part = tokens
                .next()
                .ok_or(ParseAddrError::NotEnoughParts)?
                .parse()?
        }

        if let Some(_) = tokens.next() {
            return Err(ParseAddrError::TooManyParts);
        }

        Ok(res)
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    #[debug(skip)]
    pub version: u4,

    #[debug(format = "{}")]
    pub ihl: u4,

    #[debug(format = "{:x}")]
    pub dscp: u6,

    #[debug(format = "{:b}")]
    pub ecn: u2,
    pub length: u16,

    #[debug(format = "{:04x}")]
    pub identification: u16,

    #[debug(format = "{:b}")]
    pub flags: u3,

    #[debug(format = "{}")]
    pub fragment_offset: u13,

    #[debug(format = "{}")]
    pub ttl: u8,

    #[debug(skip)]
    pub protocol: Option<Protocol>,

    #[debug(format = "{:04x}")]
    pub checksum: u16,

    pub src: Addr,
    pub dst: Addr,

    pub payload: Payload,
}

impl Packet {
    pub fn parse(i: parse::Input) -> parse::Result<Self> {
        let (i, (version, ihl)) = bits(tuple((u4::parse, u4::parse))).parse(i)?;
        assert_eq!(u8::from(version), 4);

        let (i, (dscp, ecn)) = bits(tuple((u6::parse, u2::parse))).parse(i)?;
        let (i, length) = be_u16(i)?;

        let (i, identification) = be_u16(i)?;
        let (i, (flags, fragment_offset)) = bits(tuple((u3::parse, u13::parse))).parse(i)?;

        let (i, ttl) = be_u8(i)?;
        let (i, protocol) = Protocol::parse(i)?;
        let (i, checksum) = be_u16(i)?;
        let (i, (src, dst)) = tuple((Addr::parse, Addr::parse)).parse(i)?;

        let (i, payload) = match protocol {
            Some(Protocol::ICMP) => map(icmp::Packet::parse, Payload::ICMP).parse(i)?,
            _ => (i, Payload::Unknown),
        };

        let res = Self {
            version,
            ihl,
            dscp,
            ecn,
            length,
            identification,
            flags,
            fragment_offset,
            ttl,
            protocol,
            checksum,
            src,
            dst,
            payload,
        };
        Ok((i, res))
    }

    pub fn serialize_no_checksum<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use crate::serialize::{bits, BitSerialize};
        use cf::{
            bytes::{be_u16, be_u8},
            sequence::tuple,
        };

        tuple((
            bits(move |bo| {
                // hard-code these, so we don't have to fill them out when
                // building new IPv4 packets
                let version = u4::new(4);
                let ihl = u4::new(5);
                let dscp = u6::new(0);
                let ecn = u2::new(0);

                version.write(bo);
                ihl.write(bo);
                dscp.write(bo);
                ecn.write(bo);
            }),
            be_u16(0), // length, to fill later
            be_u16(self.identification),
            bits(move |bo| {
                // again, hard-code these, we don't care about fragmentation
                // right now
                let flags = u3::new(0);
                let fragment_offset = u13::new(0);

                flags.write(bo);
                fragment_offset.write(bo);
            }),
            be_u8(self.ttl),
            // we need to do this to avoid capturing a temporary
            move |out| self.payload.protocol().serialize()(out),
            be_u16(0), // checksum, to fill later
            self.src.serialize(),
            self.dst.serialize(),
            self.payload.serialize(),
        ))
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::{
            bytes::{be_u16, le_u16},
            combinator::slice,
        };

        move |out| {
            let mut buf = cf::gen_simple(self.serialize_no_checksum(), Vec::new())?;
            let length = buf.len() as u16;
            cf::gen_simple(be_u16(length), &mut buf[2..])?;

            let header_slice = &buf[..5 * 4];
            let checksum = crate::ipv4::checksum(&header_slice);
            cf::gen_simple(le_u16(checksum), &mut buf[10..])?;

            slice(buf)(out)
        }
    }

    pub fn as_ethernet_payload(self) -> crate::ethernet::Payload {
        crate::ethernet::Payload::IPv4(self)
    }
}

impl Default for Packet {
    fn default() -> Self {
        Self {
            length: 0,
            identification: rand::random(),
            version: u4::new(4),
            ihl: u4::new(5),
            dscp: u6::new(0),
            ecn: u2::new(0),
            flags: u3::new(0),
            fragment_offset: u13::new(0),
            ttl: 128,
            protocol: None,
            checksum: 0,
            src: Addr::zero(),
            dst: Addr::zero(),
            payload: Payload::Unknown,
        }
    }
}

#[derive(Debug, TryFromPrimitive, Clone, Copy)]
#[repr(u8)]
pub enum Protocol {
    ICMP = 0x01,
    TCP = 0x06,
    UDP = 0x11,
}

impl Protocol {
    pub fn parse(i: parse::Input) -> parse::Result<Option<Self>> {
        map(be_u8, |x| Self::try_from(x).ok()).parse(i)
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        use cf::bytes::be_u8;
        be_u8(*self as u8)
    }
}

#[derive(Debug, Clone)]
pub enum Payload {
    ICMP(icmp::Packet),
    Unknown,
}

impl Payload {
    pub fn protocol(&self) -> Protocol {
        match self {
            Self::ICMP(_) => Protocol::ICMP,
            _ => unimplemented!(),
        }
    }

    pub fn serialize<'a, W: io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        move |out| match self {
            Self::ICMP(ref icmp) => icmp.serialize()(out),
            _ => unimplemented!(),
        }
    }

    pub fn as_packet(self, src: Addr, dst: Addr) -> Packet {
        Packet {
            protocol: Some(self.protocol()),
            payload: self,
            src,
            dst,
            ..Default::default()
        }
    }
}

pub fn checksum(slice: &[u8]) -> u16 {
    let (head, slice, tail) = unsafe { slice.align_to::<u16>() };
    if !head.is_empty() {
        panic!("checksum() input should be 16-bit aligned");
    }

    fn add(a: u16, b: u16) -> u16 {
        let s: u32 = (a as u32) + (b as u32);
        if s & 0x1_00_00 > 0 {
            (s + 1) as u16
        } else {
            s as u16
        }
    }

    !add(
        slice.iter().fold(0, |x, y| add(x, *y)),
        tail.iter().next().map(|&x| x as u16).unwrap_or_default(),
    )
}
