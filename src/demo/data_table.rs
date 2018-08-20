use std::io::Read;
use demo::bits::BitReader;
use demo::parse::ParseError;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct DataTables {
	pub tables: Vec<DataTable>,
	pub links: Vec<ClassLink>
}

impl DataTables {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Result<Self, ParseError> where R: Read {
		let mut tables = Vec::new();

		while bits.read_bit()? {
			tables.push(DataTable::parse(bits)?);
		}

		let mut links = Vec::with_capacity(bits.read_u16()? as usize);

		for _ in 0..links.capacity() {
			links.push(ClassLink::parse(bits)?);
		}

		Ok(DataTables { tables, links })
	}
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ClassLink {
	pub index: u16,
	pub name: String,
	pub table: String
}

impl ClassLink {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Result<Self, ParseError> where R: Read {
		let index = bits.read_u16()?;
		let name = bits.read_string()?;
		let table = bits.read_string()?;

		Ok(ClassLink { index, name, table })
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct DataTable {
	pub needs_decoder: bool,
	pub name:          String,
	pub rows:          Vec<Row>
}

impl DataTable {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Result<Self, ParseError> where R: Read {
		let needs_decoder = bits.read_bit()?;
		let name = bits.read_string()?;
		let entries = bits.read_bits(10)?;

		let mut rows = Vec::with_capacity(entries as usize);

		for _ in 0..entries {
			rows.push(Row::parse(bits)?);
		}

		Ok(DataTable {
			needs_decoder,
			name,
			rows
		})
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Row {
	pub name:  String,
	pub flags: Flags,
	pub data:  RowData
}

impl Row {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Result<Self, ParseError> where R: Read {
		let kind_id = bits.read_bits(5)?;
		let kind = RowKind::from_id(kind_id).ok_or(ParseError::BadEnumIndex { name: "data_table::RowKind", value: kind_id})?;

		let name = bits.read_string()?;
		let flags = Flags(bits.read_u16()?);

		let data = if kind == RowKind::Table {
			let name = bits.read_string()?;

			RowData::Table { name }
		} else if flags.has(Flag::Exclude) {
			let exclusion = bits.read_string()?;

			RowData::Exclude { exclusion }
		} else if kind == RowKind::Array {
			let max_elements = bits.read_bits(10)? as u16;

			RowData::Array { max_elements }
		} else {
			let low = bits.read_f32()?;
			let high = bits.read_f32()?;
			let bits = bits.read_bits(7)? as u8;

			match kind {
				RowKind::Integer => RowData::Integer { bits },
				RowKind::Float   => RowData::Float   { low, high, bits },
				RowKind::Vec3    => RowData::Vec3    { low, high, bits },
				RowKind::Vec2    => RowData::Vec2    { low, high, bits },
				RowKind::String  => RowData::String,
				RowKind::Array   => unreachable!(),
				RowKind::Table   => unreachable!()
			}
		};

		Ok(Row { name, flags, data })
	}
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum Flag {
	/// Integer: Unsigned instead of Signed
	Unsigned,
	/// Float / Vector: Encoded like a coordinate.
	/// Bit count ignored.
	Coordinate,
	/// Float: Don't scale the value into the provided range.
	NoScale,
	/// Float: Limit maximum to range minus 1 bit unit
	RoundDown,
	/// Float: Limit minimum to range minus 1 bit unit
	RoundUp,
	/// Vector: Treat vector as if it is normalized
	Normal,
	/// Any: Points to another property to be excluded.
	Exclude,
	/// Vector: Encode with Xyz/Exponent encoding.
	XyzeEncoding,
	/// Any: Property is contained in an array, so don't put it into the flattened rows.
	InsideArray,
	/// Any: Always send the data table.
	AlwaysSend,
	/// Any: This property changes often, and gets a smaller, more compressible index for optimization.
	ChangesOften,
	/// ???
	VectorElem,
	/// Offset is 0 and "doesn't change pointer" ???
	Collapsible,
	/// Float/Vector: CoordinateMP encoding - Coordinate encoding with special treatment in multiplayer.
	CoordinateMp,
	/// Float/Vector: CoordinateMP encoding with low precision. (3 fractional bits instead of 5)
	CoordinateMpLowPrecision,
	/// Float/Vector: CoordinateMP encoding with values rounded to integers
	CoordinateMpIntegral,

}

impl Flag {
	fn from_index(index: u8) -> Option<Self> {
		Some(match index {
			0 => Flag::Unsigned,
			1 => Flag::Coordinate,
			2 => Flag::NoScale,
			3 => Flag::RoundDown,
			4 => Flag::RoundUp,
			5 => Flag::Normal,
			6 => Flag::Exclude,
			7 => Flag::XyzeEncoding,
			8 => Flag::InsideArray,
			9 => Flag::AlwaysSend,
			10 => Flag::ChangesOften,
			11 => Flag::VectorElem,
			12 => Flag::Collapsible,
			13 => Flag::CoordinateMp,
			14 => Flag::CoordinateMpIntegral,
			15 => Flag::CoordinateMpLowPrecision,
			_ => return None
		})
	}
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct Flags(pub u16);

impl Flags {
	pub fn has(&self, flag: Flag) -> bool {
		(self.0 >> (flag as u8)) & 1 == 1
	}

	pub fn iter(self) -> FlagsIter {
		FlagsIter { flags: self, index: 0 }
	}
}

impl Display for Flags {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		let mut last_flag = None;

		for flag in self.iter() {
			if let Some(last_flag) = last_flag {
				write!(f, "{:?}, ", last_flag)?;
			}

			last_flag = Some(flag);
		}

		if let Some(last_flag) = last_flag {
			write!(f, "{:?}", last_flag)
		} else {
			write!(f, "(none)")
		}
	}
}

pub struct FlagsIter {
	flags: Flags,
	index: u8
}

impl FlagsIter {
	fn try_next(&mut self) -> Option<Flag> {
		let flag = Flag::from_index(self.index).unwrap();
		self.index += 1;

		if self.flags.has(flag) {
			Some(flag)
		} else {
			None
		}
	}
}

impl Iterator for FlagsIter {
	type Item = Flag;

	fn next(&mut self) -> Option<Flag> {
		while self.index < 16 {
			if let Some(flag) = self.try_next() {
				return Some(flag);
			}
		}

		None
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum RowData {
	Exclude { exclusion: String },
	Integer { bits: u8 },
	Float   { low: f32, high: f32, bits: u8 },
	Vec3    { low: f32, high: f32, bits: u8 },
	Vec2    { low: f32, high: f32, bits: u8 },
	String,
	Array   { max_elements: u16 },
	Table   { name: String }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum RowKind {
	Integer,
	Float,
	Vec3,
	Vec2,
	String,
	Array,
	Table
}

impl RowKind {
	pub fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => RowKind::Integer,
			1 => RowKind::Float,
			2 => RowKind::Vec3,
			3 => RowKind::Vec2,
			4 => RowKind::String,
			5 => RowKind::Array,
			6 => RowKind::Table,
			_ => return None
		})
	}
}