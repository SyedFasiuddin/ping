use nom::error::{ErrorKind as NomErrorKind, ParseError as NomParseError};
use nom::{ErrorConvert, Slice};
use std::ops::RangeFrom;
use ux::*;

pub type Input<'a> = &'a [u8];
pub type Result<'a, T> = nom::IResult<Input<'a>, T, Error<Input<'a>>>;

pub type BitInput<'a> = (&'a [u8], usize);
pub type BitResult<'a, T> = nom::IResult<BitInput<'a>, T, Error<BitInput<'a>>>;

#[derive(Debug)]
pub struct Error<I> {
    pub errors: Vec<(I, NomErrorKind)>,
}

impl<I> NomParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: NomErrorKind) -> Self {
        let errors = vec![(input, kind)];
        Self { errors }
    }

    fn append(input: I, kind: NomErrorKind, mut other: Self) -> Self {
        other.errors.push((input, kind));
        other
    }
}

impl<I> ErrorConvert<Error<I>> for Error<(I, usize)>
where
    I: Slice<RangeFrom<usize>>,
{
    fn convert(self) -> Error<I> {
        let errors = self
            .errors
            .into_iter()
            .map(|((rest, offset), err)| (rest.slice(offset / 8..), err))
            .collect();
        Error { errors }
    }
}

pub trait BitParsable
where
    Self: Sized,
{
    fn parse(i: BitInput) -> BitResult<Self>;
}

macro_rules! impl_bit_parsable_for_ux {
    ($($width:expr),*) => {
        $(
            paste::paste! {
                impl BitParsable for [<u $width>] {
                    fn parse(i: BitInput) -> BitResult<Self> {
                        use nom::{bits::complete::take as take_bits, combinator::map, Parser};

                        map(take_bits($width as usize), Self::new).parse(i)
                    }
                }
            }
        )*
    };
}

impl_bit_parsable_for_ux!(2, 3, 4, 6, 13);
