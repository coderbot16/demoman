use std::io::Read;
use byteorder::{ReadBytesExt, LittleEndian};
use demo::bits::BitReader;
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
	pub fn parse<R>(input: &mut R) -> Self where R: Read {
		let kind = FrameKind::from_id(input.read_u8().unwrap()).unwrap();

		let tick = if kind == FrameKind::Stop {
			(input.read_u16::<LittleEndian>().unwrap() as u32) | ((input.read_u8().unwrap() as u32) << 16)
		} else {
			input.read_u32::<LittleEndian>().unwrap()
		};

		let payload = match kind {
			FrameKind::Unused         => panic!("FrameKind::Unused in demo file"),
			FrameKind::SignonUpdate   => FramePayload::SignonUpdate(Update::parse(input)),
			FrameKind::Update         => FramePayload::Update(Update::parse(input)),
			FrameKind::TickSync       => FramePayload::TickSync,
			FrameKind::ConsoleCommand => {
				let len = input.read_u32::<LittleEndian>().unwrap();

				let mut data = Vec::with_capacity(len as usize);

				for _ in 0..len {
					let value = input.read_u8().unwrap();

					if value == 0 {
						break;
					}

					data.push(value);
				}

				FramePayload::ConsoleCommand(String::from_utf8(data).unwrap())
			},
			FrameKind::UserCmdDelta => {
				let delta = UserCmdDelta::parse(input).unwrap();

				FramePayload::UserCmdDelta(delta)
			},
			FrameKind::DataTables => {
				let len = input.read_u32::<LittleEndian>().unwrap();
				let mut bits = BitReader::new(input, len as usize).unwrap();

				let tables = DataTables::parse(&mut bits);
				assert_eq!(bits.unread_bytes(), 0);

				FramePayload::DataTables(tables.unwrap())
			},
			FrameKind::Stop => FramePayload::Stop,
			FrameKind::StringTables => {
				let len = input.read_u32::<LittleEndian>().unwrap();
				let mut bits = BitReader::new(input, len as usize).unwrap();

				let tables = StringTables::parse(&mut bits);
				assert_eq!(bits.unread_bytes(), 0);

				FramePayload::StringTables(tables.unwrap())
			}
		};

		Frame { tick, payload }
	}
}

#[derive(Debug, Clone)]
pub enum FramePayload {
	SignonUpdate(Update),
	Update(Update),
	TickSync,
	ConsoleCommand(String),
	UserCmdDelta(UserCmdDelta),
	DataTables(DataTables),
	Stop,
	StringTables(StringTables)
}

impl FramePayload {
	pub fn kind(&self) -> FrameKind {
		match self {
			&FramePayload::SignonUpdate(_)   => FrameKind::SignonUpdate,
			&FramePayload::Update(_)         => FrameKind::Update,
			&FramePayload::TickSync          => FrameKind::TickSync,
			&FramePayload::ConsoleCommand(_) => FrameKind::ConsoleCommand,
			&FramePayload::UserCmdDelta(_)   => FrameKind::UserCmdDelta,
			&FramePayload::DataTables(_)     => FrameKind::DataTables,
			&FramePayload::Stop              => FrameKind::Stop,
			&FramePayload::StringTables(_)   => FrameKind::StringTables,
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
	pub fn parse<R>(input: &mut R) -> Self where R: Read {
		let position = PositionUpdate::parse(input).unwrap();
		let sequence_in = input.read_u32::<LittleEndian>().unwrap();
		let sequence_out = input.read_u32::<LittleEndian>().unwrap();

		let len = input.read_u32::<LittleEndian>().unwrap();

		let mut packets = Vec::with_capacity(len as usize);
		for _ in 0..len {
			packets.push(input.read_u8().unwrap());
		}

		Update { position, sequence_in, sequence_out, packets }
	}
}
