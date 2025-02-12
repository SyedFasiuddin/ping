use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use cookie_factory as cf;
use std::io;
use ux::*;

pub type BitOutput = BitVec<u8, Msb0>;

pub trait WriteLastNBits {
    fn write_last_n_bits(&mut self, b: &BitSlice<u16, Msb0>, num_bits: usize);
}

impl WriteLastNBits for BitOutput {
    fn write_last_n_bits(&mut self, b: &BitSlice<u16, Msb0>, num_bits: usize) {
        let start = b.len() - num_bits;
        self.extend_from_bitslice(&b[start..])
    }
}

pub fn bits<W, F>(f: F) -> impl cf::SerializeFn<W>
where
    W: io::Write,
    F: Fn(&mut BitOutput),
{
    move |mut out: cf::WriteContext<W>| {
        let mut bo = BitOutput::new();
        f(&mut bo);

        io::Write::write(&mut out, &bo.into_vec())?;
        Ok(out)
    }
}

pub trait BitSerialize {
    fn write(&self, b: &mut BitOutput);
}

macro_rules! impl_bit_serialize_for_ux {
    ($($width:expr),*) => {
        $(
            paste::paste! {
                impl BitSerialize for [<u $width>] {
                    fn write(&self, b: &mut BitOutput) {
                        let bits = BitVec::<_, Msb0>::from_element(u16::from(*self));
                        b.write_last_n_bits(bits.as_bitslice(), $width);
                    }
                }
            }
        )*
    };
}

impl_bit_serialize_for_ux!(2, 3, 4, 6, 13);
