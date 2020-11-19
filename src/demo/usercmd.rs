use std::io::{self, Read};
use crate::demo::bits::BitReader;
use crate::demo::parse::ParseError;
use crate::demo::bytes::Reader;
use std::convert::TryInto;

/// Delta encoded UserCmd.
/// None values represent that the value did not change.
#[derive(Debug, Copy, Clone)]
pub struct UserCmdDelta {
	/// If None, then command_number is the last number + 1.
	command_number: Option<u32>,
	/// If None, then tick_count is the last count + 1.
	tick_count: Option<u32>,
	view_angles: (Option<f32>, Option<f32>, Option<f32>),
	forward: Option<f32>,
	side: Option<f32>,
	up: Option<f32>,
	buttons: Option<u32>,
	impulse: Option<u8>,
	weapon_select: Option<(u16, Option<u8>)>, // 11 bits, 6 bits
	mouse_delta: (Option<i16>, Option<i16>)
}

impl UserCmdDelta {
	pub fn parse(reader: &mut BitReader) -> Result<Self, ParseError> {
		Ok(UserCmdDelta {
			command_number: if reader.read_bit()? { Some(reader.read_u32()?) } else { None },
			tick_count:     if reader.read_bit()? { Some(reader.read_u32()?) } else { None },
			view_angles: (
				if reader.read_bit()? { Some(reader.read_f32()?) } else { None },
				if reader.read_bit()? { Some(reader.read_f32()?) } else { None },
				if reader.read_bit()? { Some(reader.read_f32()?) } else { None }
			),
			forward: if reader.read_bit()? { Some(reader.read_f32()?) } else { None },
			side: if reader.read_bit()? { Some(reader.read_f32()?) } else { None },
			up: if reader.read_bit()? { Some(reader.read_f32()?) } else { None },
			buttons: if reader.read_bit()? { Some(reader.read_u32()?) } else { None },
			impulse: if reader.read_bit()? { Some(reader.read_u8()?) } else { None },
			weapon_select: if reader.read_bit()? {
				Some((
					reader.read_bits(11)? as u16,
					if reader.read_bit()? { Some(reader.read_bits(6)? as u8) } else { None }
				))
			} else {
				None
			},
			mouse_delta: (
				if reader.read_bit()? { Some(reader.read_i16()?) } else { None },
				if reader.read_bit()? { Some(reader.read_i16()?) } else { None }
			)
		})
	}
}

#[derive(Debug, Copy, Clone)]
pub struct PositionUpdate {
	pub flags:     u32,
	pub original:  Position,
	pub resampled: Position
}

impl PositionUpdate {
	// u32 + 2x Position
	pub const LEN: usize = 4 + Position::LEN * 2;

	pub fn read<R: Read>(input: &mut R) -> io::Result<Self> {
		let mut bytes = [0u8; Self::LEN];

		input.read_exact(&mut bytes)?;

		Ok(Self::from_bytes(bytes))
	}

	pub fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
		let mut reader = Reader::new(&bytes);

		PositionUpdate {
			flags:     reader.u32(),
			original:  Position::read(&mut reader),
			resampled: Position::read(&mut reader)
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
	pub view_orgin:        (f32, f32, f32),
	pub view_angles:       (f32, f32, f32),
	pub view_angles_local: (f32, f32, f32),
}

impl Position {
	// 4 bytes per float * 3 floats per vector * 3 vectors
	pub const LEN: usize = 4 * 3 * 3;

	fn read(reader: &mut Reader) -> Self {
		// Infallible as long as the reader has sufficient bytes
		Self::from_bytes(reader.bytes(Self::LEN).try_into().unwrap())
	}

	pub fn from_bytes(bytes: &[u8; Self::LEN]) -> Self {
		let mut reader = Reader::new(bytes);

		Position {
			view_orgin:        (reader.f32(), reader.f32(), reader.f32()),
			view_angles:       (reader.f32(), reader.f32(), reader.f32()),
			view_angles_local: (reader.f32(), reader.f32(), reader.f32())
		}
	}
}