use std::io;
use std::string::FromUtf8Error;
use crate::demo::bits::{BitReader, InsufficientBits, ReadStringError};
use std::borrow::Cow;

#[derive(Debug)]
pub enum Needed {
	Bits { requested: usize, available: usize },
	Bytes { requested: usize, available: usize }
}

#[derive(Debug)]
pub enum ParseError {
	InsufficientBits(InsufficientBits),
	Needed(Needed),
	Utf8(FromUtf8Error),
	Io(io::Error),
	BadEnumIndex { name: &'static str, value: u32 },
	OutOfBounds { name: &'static str, value: u32, min: u32, max: u32 },
	Custom(Cow<'static, str>)
}

impl From<InsufficientBits> for ParseError {
	fn from(from: InsufficientBits) -> Self {
		ParseError::InsufficientBits(from)
	}
}

impl From<ReadStringError> for ParseError {
	fn from(from: ReadStringError) -> Self {
		match from {
			ReadStringError::InsufficientBits(insufficient) =>
				ParseError::InsufficientBits(insufficient),
			ReadStringError::Utf8(utf8) => ParseError::Utf8(utf8)
		}
	}
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
	fn parse(bits: &mut BitReader) -> Result<Self, ParseError>;
}