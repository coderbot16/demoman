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

fn read_u8_array<R>(input: &mut R) -> Result<Vec<u8>, io::Error> where R: Read {
	let len = input.read_u32::<LittleEndian>()?;

	let mut buf = vec![0; len as usize];

	input.read_exact(&mut buf)?;

	Ok(buf)
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

		Frame::parse_with_kind(input, kind)
	}

	pub fn parse_with_kind<R>(input: &mut R, kind: FrameKind) -> Result<Self, ParseError> where R: Read {
		let tick = if kind == FrameKind::Stop {
			(input.read_u16::<LittleEndian>()? as u32) | ((input.read_u8()? as u32) << 16)
		} else {
			input.read_u32::<LittleEndian>()?
		};

		let payload = match kind {
			FrameKind::SignonUpdate   => FramePayload::SignonUpdate(Update::parse(input)?),
			FrameKind::Update         => FramePayload::Update(Update::parse(input)?),
			FrameKind::TickSync       => FramePayload::TickSync,
			FrameKind::ConsoleCommand => {
				let mut buf = read_u8_array(input)?;

				let mut terminator = None;
				for (index, &byte) in buf.iter().enumerate() {
					if byte == 0 {
						terminator = Some(index);
						break;
					}
				}

				if let Some(terminator) = terminator {
					for _ in 0..(buf.len() - terminator) {
						buf.pop();
					}
				}

				FramePayload::ConsoleCommand(String::from_utf8(buf)?)
			},
			FrameKind::UserCmdDelta => FramePayload::UserCmdDelta {
				sequence: input.read_u32::<LittleEndian>()?,
				frame: UserCmdFrame::from_raw(read_u8_array(input)?)
			},
			FrameKind::DataTables => FramePayload::DataTables(DataTablesFrame::from_raw(read_u8_array(input)?)),
			FrameKind::Stop => FramePayload::Stop,
			FrameKind::StringTables => FramePayload::StringTables(StringTablesFrame::from_raw(read_u8_array(input)?))
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
	UserCmdDelta { sequence: u32, frame: UserCmdFrame },
	DataTables(DataTablesFrame),
	Stop,
	StringTables(StringTablesFrame)
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
		input.read_exact(&mut packets)?;

		Ok(Update { position, sequence_in, sequence_out, packets })
	}
}

#[derive(Debug, Clone)]
pub struct DataTablesFrame(Vec<u8>);
impl DataTablesFrame {
	pub fn from_raw(data: Vec<u8>) -> Self {
		DataTablesFrame(data)
	}

	pub fn parse(&self) -> Result<DataTables, ParseError> {
		let mut bits = BitReader::new(&self.0);

		let tables = DataTables::parse(&mut bits)?;
		assert_eq!(bits.unread_bytes(), 0);

		Ok(tables)
	}

	pub fn raw(&self) -> &[u8] {
		&self.0
	}

	pub fn into_raw(self) -> Vec<u8> {
		self.0
	}
}

#[derive(Debug, Clone)]
pub struct StringTablesFrame(Vec<u8>);
impl StringTablesFrame {
	pub fn from_raw(data: Vec<u8>) -> Self {
		StringTablesFrame(data)
	}

	pub fn parse(&self) -> Result<StringTables, ParseError> {
		let mut bits = BitReader::new(&self.0);

		let tables = StringTables::parse(&mut bits)?;
		assert_eq!(bits.unread_bytes(), 0);

		Ok(tables)
	}

	pub fn raw(&self) -> &[u8] {
		&self.0
	}

	pub fn into_raw(self) -> Vec<u8> {
		self.0
	}
}

#[derive(Debug, Clone)]
pub struct UserCmdFrame(Vec<u8>);
impl UserCmdFrame {
	pub fn from_raw(data: Vec<u8>) -> Self {
		UserCmdFrame(data)
	}

	pub fn parse(&self) -> Result<UserCmdDelta, ParseError> {
		let mut bits = BitReader::new(&self.0);

		let tables = UserCmdDelta::parse(&mut bits)?;
		assert_eq!(bits.unread_bytes(), 0);

		Ok(tables)
	}

	pub fn raw(&self) -> &[u8] {
		&self.0
	}

	pub fn into_raw(self) -> Vec<u8> {
		self.0
	}
}