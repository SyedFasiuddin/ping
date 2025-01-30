use std::fmt;
use std::num::ParseIntError;

#[repr(C)]
pub struct Addr(pub [u8; 4]);

impl fmt::Debug for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d] = self.0;
        write!(f, "{}.{}.{}.{}", a, b, c, d)
    }
}

#[derive(Debug)]
pub enum ParseAddrError {
    NotEnoughParts,
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for ParseAddrError {
    fn from(e: ParseIntError) -> Self {
        ParseAddrError::ParseIntError(e)
    }
}

impl Addr {
    pub fn parse<S>(s: S) -> Result<Self, ParseAddrError>
    where
        S: AsRef<str>,
    {
        let mut tokens = s.as_ref().split(".");

        let mut f = || -> Result<u8, ParseAddrError> {
            Ok(tokens
                .next()
                .ok_or(ParseAddrError::NotEnoughParts)?
                .parse()?)
        };

        Ok(Self([f()?, f()?, f()?, f()?]))
    }
}
