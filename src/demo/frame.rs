use std::io::{self, Read};
use bitstream::BitReader;
use crate::demo::parse::ParseError;
use crate::demo::data_table::DataTables;
use crate::demo::usercmd::{UserCmdDelta, PositionUpdate};
use crate::packets::string_table::StringTables;

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

fn read_u8<R>(input: &mut R) -> Result<u8, io::Error> where R: Read {
	let mut byte = [0u8; 1];

	input.read_exact(&mut byte)?;

	Ok(byte[0])
}

fn read_u24<R>(input: &mut R) -> Result<u32, io::Error> where R: Read {
	let mut bytes = [0u8; 4];

	// Only read the first 3 bytes, so that the last byte is empty
	// Since this is little-endian byte order, the last byte being empty means that
	// the most significant byte (x << 24) will be zero, which is what we'd expect
	// for a 24-bit value stored in a 32-bit data type.
	input.read_exact(&mut bytes[0..3])?;

	Ok(u32::from_le_bytes(bytes))
}

fn read_u32<R>(input: &mut R) -> Result<u32, io::Error> where R: Read {
	let mut bytes = [0u8; 4];

	input.read_exact(&mut bytes)?;

	Ok(u32::from_le_bytes(bytes))
}

fn read_u8_array<R>(input: &mut R) -> Result<Vec<u8>, io::Error> where R: Read {
	let len = read_u32(input)?;

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
		let kind_id = read_u8(input)?;
		let kind = FrameKind::from_id(kind_id).ok_or(ParseError::BadEnumIndex { name: "FrameKind", value: u32::from(kind_id) })?;

		Frame::parse_with_kind(input, kind)
	}

	pub fn parse_with_kind<R>(input: &mut R, kind: FrameKind) -> Result<Self, ParseError> where R: Read {
		let tick = if kind == FrameKind::Stop {
			read_u24(input)?
		} else {
			read_u32(input)?
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
				sequence: read_u32(input)?,
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
		Ok(Update {
			position: PositionUpdate::read(input)?,
			sequence_in: read_u32(input)?,
			sequence_out: read_u32(input)?,
			packets: read_u8_array(input)?
		})
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