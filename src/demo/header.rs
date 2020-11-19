use std::fmt::{Debug, Formatter, Error};
use std::str::{self, Utf8Error};

pub const PATH_LENGTH: usize = 260;
pub const HEADER_LENGTH: usize = 8 + 4 + 4 + PATH_LENGTH + PATH_LENGTH + PATH_LENGTH + PATH_LENGTH + 4 + 4 + 4 + 4; // 1072

pub struct HeaderString([u8; 260]);

pub struct HeaderStr<'s>(& 's [u8; 260]);
impl<'s> HeaderStr<'s> {
	pub fn from_slice(slice: &'s [u8]) -> Option<Self> {
		if slice.len() != 260 {
			None
		} else {
			Some(HeaderStr(array_ref![slice, 0, 260]))
		}
	}

	pub fn bytes(&self) -> &[u8; 260] {
		&self.0
	}

	pub fn str_bytes(&self) -> &[u8] {
		let mut len = 260;

		for (index, &byte) in (self.0 as &[u8]).iter().enumerate() {
			if byte == 0 {
				len = index;
				break;
			}
		}

		&self.0[..len]
	}

	pub fn to_str(&self) -> Result<&str, Utf8Error> {
		str::from_utf8(self.str_bytes())
	}
}

impl<'s> Debug for HeaderStr<'s> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		if let Ok(utf8) = self.to_str() {
			write!(f, "{:?}", utf8)
		} else {
			write!(f, "InvalidUtf8({:?})", self.str_bytes())
		}
	}
}

#[derive(Debug)]
pub struct DemoHeader<'data> {
	pub demo_protocol:    i32,
	pub network_protocol: i32,
	pub server_name:      HeaderStr<'data>,
	pub client_name:      HeaderStr<'data>,
	pub map_name:         HeaderStr<'data>,
	pub game_directory:   HeaderStr<'data>,
	pub playback_seconds: f32,
	pub ticks:            i32,
	pub frames:           i32,
	pub signon_length:    i32
}

impl<'data> DemoHeader<'data> {
	pub fn parse(data: &'data [u8; HEADER_LENGTH]) -> Result<DemoHeader, IncorrectMagic> {
		let mut reader  = Reader {
			bytes: data
		};

		let magic = reader.grab(8);

		if magic !=  b"HL2DEMO\0" {
			return Err(IncorrectMagic(magic));
		}

		Ok(DemoHeader {
			demo_protocol: reader.i32(),
			network_protocol: reader.i32(),
			server_name: reader.str(),
			client_name: reader.str(),
			map_name: reader.str(),
			game_directory: reader.str(),
			playback_seconds: reader.f32(),
			ticks: reader.i32(),
			frames: reader.i32(),
			signon_length: reader.i32()
		})
	}
}

#[derive(Debug)]
pub struct IncorrectMagic<'a>(&'a [u8]);

struct Reader<'a> {
	bytes: &'a [u8]
}

impl<'a> Reader<'a> {
	fn grab(&mut self, len: usize) -> &'a [u8] {
		let (requested, rest) = self.bytes.split_at(len);
		self.bytes = rest;

		requested
	}

	fn str(&mut self) -> HeaderStr<'a> {
		HeaderStr::from_slice(self.grab(260)).unwrap()
	}

	fn u32(&mut self) -> u32 {
		if let &[a, b, c, d] = self.grab(4) {
			u32::from_le_bytes([a, b, c, d])
		} else {
			unreachable!()
		}
	}

	fn i32(&mut self) -> i32 {
		self.u32() as i32
	}

	fn f32(&mut self) -> f32 {
		f32::from_bits(self.u32())
	}
}
