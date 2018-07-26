use ::nom::IResult;
use std::fmt::{Debug, Formatter, Error};
use std::str::{self, Utf8Error};
use nom::{le_i32, le_f32};

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
	pub fn parse(data: &'data [u8]) -> IResult<&'data [u8], DemoHeader<'data>, u32> {
		parse_demo_header(data)
	}
}

named!(pub parse_demo_header<DemoHeader>,
	do_parse!(
		tag!(b"HL2DEMO\0") >>
		demo_protocol: le_i32 >>
		network_protocol: le_i32 >>
		server_name: take!(PATH_LENGTH) >>
		client_name: take!(PATH_LENGTH) >>
		map_name: take!(PATH_LENGTH) >>
		game_directory: take!(PATH_LENGTH) >>
		playback_seconds: le_f32 >>
		ticks: le_i32 >>
		frames: le_i32 >>
		signon_length: le_i32 >>
		(DemoHeader {
				demo_protocol:    demo_protocol,
				network_protocol: network_protocol,
				server_name:      HeaderStr::from_slice(server_name).unwrap(),
				client_name:      HeaderStr::from_slice(client_name).unwrap(),
				map_name:         HeaderStr::from_slice(map_name).unwrap(),
				game_directory:   HeaderStr::from_slice(game_directory).unwrap(),
				playback_seconds: playback_seconds,
				ticks:            ticks,
				frames:           frames,
				signon_length:    signon_length
		})
	)
);