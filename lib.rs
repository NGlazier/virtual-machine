//! The Grumpy compiler.

#![warn(clippy::all)]
use std::{error, fmt, io, num};

// Declare modules in the grumpy crate.
pub mod assemble;
pub mod isa;
pub mod vm;

/// Trait for types that can be serialized to a binary representation.
pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

/// Trait for types that can be deserialized from a binary representation.
pub trait FromBytes : Sized {
    type Err;
    fn from_bytes<T: Iterator<Item=u8>>(v: &mut T) -> Result<Self, Self::Err>;
}

/// A type for parse errors.
#[derive(Debug)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for ParseError {}

impl From<num::ParseIntError> for ParseError {
    fn from(err: num::ParseIntError) -> Self {
        ParseError(format!("{}", err))
    }
}

impl From<ParseError> for io::Error {
    fn from(err: ParseError) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("{:?}", err))
    }
}

