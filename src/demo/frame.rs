use std::io::{self, Read};
use byteorder::{ReadBytesExt, LittleEndian};
use demo::bits::BitReader;
use demo::parse::ParseError;
use demo::data_table::DataTables;
use demo::usercmd::{UserCmdDelta, PositionUpdate};
use packets::string_table::StringTables;

// TODO: NetProto 36+: CustomData frame

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum FrameKind {
	Unused,
	SignonUpdate,
	Update,
	TickSync,
	ConsoleCommand,
	UserCmdDelta,
	DataTables,
	Stop,
	StringTables
}

impl FrameKind {
	fn from_id(id: u8) -> Option<Self> {
		Some(match id {
			0 => FrameKind::Unused,
			1 => FrameKind::SignonUpdate,
			2 => FrameKind::Update,
			3 => FrameKind::TickSync,
			4 => FrameKind::ConsoleCommand,
			5 => FrameKind::UserCmdDelta,
			6 => FrameKind::DataTables,
			7 => FrameKind::Stop,
			8 => FrameKind::StringTables,
			_ => return None
		})
	}
}

#[derive(Debug, Clone)]
pub struct Frame {
	pub tick: u32,
	pub payload: FramePayload
}

impl Frame {
	pub fn parse<R>(input: &mut R) -> Result<Self, ParseError> where R: Read {
		let kind_id = input.read_u8()?;
		let kind = FrameKind::from_id(kind_id).ok_or(ParseError::BadEnumIndex { name: "FrameKind", value: u32::from(kind_id) })?;

		let tick = if kind == FrameKind::Stop {
			(input.read_u16::<LittleEndian>()? as u32) | ((input.read_u8()? as u32) << 16)
		} else {
			input.read_u32::<LittleEndian>()?
		};

		let payload = match kind {
			FrameKind::Unused         => return Err(ParseError::BadEnumIndex { name: "FrameKind", value: 0 }),
			FrameKind::SignonUpdate   => FramePayload::SignonUpdate(Update::parse(input)?),
			FrameKind::Update         => FramePayload::Update(Update::parse(input)?),
			FrameKind::TickSync       => FramePayload::TickSync,
			FrameKind::ConsoleCommand => {
				let len = input.read_u32::<LittleEndian>()?;

				let mut data = Vec::with_capacity(len as usize);

				for _ in 0..len {
					let value = input.read_u8()?;

					if value == 0 {
						break;
					}

					data.push(value);
				}

				FramePayload::ConsoleCommand(String::from_utf8(data)?)
			},
			FrameKind::UserCmdDelta => {
				let sequence = input.read_u32::<LittleEndian>()?;

				let len = input.read_u32::<LittleEndian>()?;
				let mut buf = vec![0; len as usize];
				input.read_exact(&mut buf);

				let mut bits = BitReader::new(&buf);

				let delta = UserCmdDelta::parse(&mut bits)?;

				FramePayload::UserCmdDelta { sequence, delta }
			},
			FrameKind::DataTables => {
				let len = input.read_u32::<LittleEndian>()?;
				let mut buf = vec![0; len as usize];
				input.read_exact(&mut buf);

				let mut bits = BitReader::new(&buf);

				let tables = DataTables::parse(&mut bits)?;
				assert_eq!(bits.unread_bytes(), 0);

				FramePayload::DataTables(tables)
			},
			FrameKind::Stop => FramePayload::Stop,
			FrameKind::StringTables => {
				let len = input.read_u32::<LittleEndian>()?;
				let mut buf = vec![0; len as usize];
				input.read_exact(&mut buf);

				let mut bits = BitReader::new(&buf);

				let tables = StringTables::parse(&mut bits)?;
				assert_eq!(bits.unread_bytes(), 0);

				FramePayload::StringTables(tables)
			}
		};

		Ok(Frame { tick, payload })
	}
}

#[derive(Debug, Clone)]
pub enum FramePayload {
	SignonUpdate(Update),
	Update(Update),
	TickSync,
	ConsoleCommand(String),
	UserCmdDelta { sequence: u32, delta: UserCmdDelta },
	DataTables(DataTables),
	Stop,
	StringTables(StringTables)
}

impl FramePayload {
	pub fn kind(&self) -> FrameKind {
		match self {
			&FramePayload::SignonUpdate(_)     => FrameKind::SignonUpdate,
			&FramePayload::Update(_)           => FrameKind::Update,
			&FramePayload::TickSync            => FrameKind::TickSync,
			&FramePayload::ConsoleCommand(_)   => FrameKind::ConsoleCommand,
			&FramePayload::UserCmdDelta { .. } => FrameKind::UserCmdDelta,
			&FramePayload::DataTables(_)       => FrameKind::DataTables,
			&FramePayload::Stop                => FrameKind::Stop,
			&FramePayload::StringTables(_)     => FrameKind::StringTables,
		}
	}
}

#[derive(Debug, Clone)]
pub struct Update {
	pub position: PositionUpdate,
	pub sequence_in: u32,
	pub sequence_out: u32,
	pub packets: Vec<u8>
}

impl Update {
	pub fn parse<R>(input: &mut R) -> Result<Self, io::Error> where R: Read {
		let position = PositionUpdate::parse(input)?;
		let sequence_in = input.read_u32::<LittleEndian>()?;
		let sequence_out = input.read_u32::<LittleEndian>()?;

		let len = input.read_u32::<LittleEndian>()?;
		let mut packets = vec![0; len as usize];
		input.read_exact(&mut packets);

		Ok(Update { position, sequence_in, sequence_out, packets })
	}
}
