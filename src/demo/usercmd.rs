use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Read, Cursor};
use demo::bits::BitReader;
use demo::parse::ParseError;

/// Delta encoded UserCmd.
/// None values represent that the value did not change.
#[derive(Debug, Copy, Clone)]
pub struct UserCmdDelta {
	pub sequence: u32,
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
	pub fn parse<R: Read>(input: &mut R) -> Result<Self, ParseError> {
		let sequence = input.read_u32::<LittleEndian>()?;
		let len = input.read_u32::<LittleEndian>()?;

		let mut data = Vec::with_capacity(len as usize);

		for _ in 0..len {
			data.push(input.read_u8().unwrap());
		}

		let mut cursor = Cursor::new(&mut data);
		let mut reader = BitReader::new(&mut cursor, len as usize)?;

		Ok(UserCmdDelta {
			sequence,
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
	pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
		Ok(PositionUpdate {
			flags:     input.read_u32::<LittleEndian>()?,
			original:  Position::parse(input)?,
			resampled: Position::parse(input)?
		})
	}
}

#[derive(Debug, Copy, Clone)]
pub struct Position {
	pub view_orgin:        (f32, f32, f32),
	pub view_angles:       (f32, f32, f32),
	pub view_angles_local: (f32, f32, f32),
}

impl Position {
	pub fn parse<R: Read>(input: &mut R) -> io::Result<Self> {
		Ok(Position {
			view_orgin:        (input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?),
			view_angles:       (input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?),
			view_angles_local: (input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?, input.read_f32::<LittleEndian>()?)
		})
	}
}