use std::io;
use std::string::FromUtf8Error;
use demo::bits::BitReader;

#[derive(Debug)]
pub enum Needed {
	Bits { requested: usize, available: usize },
	Bytes { requested: usize, available: usize }
}

#[derive(Debug)]
pub enum ParseError {
	Needed(Needed),
	Utf8(FromUtf8Error),
	Io(io::Error)
}

impl From<Needed> for ParseError {
	fn from(from: Needed) -> Self {
		ParseError::Needed(from)
	}
}

impl From<FromUtf8Error> for ParseError {
	fn from(from: FromUtf8Error) -> Self {
		ParseError::Utf8(from)
	}
}

impl From<io::Error> for ParseError {
	fn from(from: io::Error) -> Self {
		ParseError::Io(from)
	}
}

pub trait Encode: Sized {
	fn parse<R>(bits: &mut BitReader<R>) -> Result<Self, ParseError> where R: io::Read;
}