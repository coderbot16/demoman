use demo::bits::BitReader;
use demo::parse::ParseError;
use std::io::Read;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct GameEventList(pub Vec<GameEventInfo>);

impl GameEventList {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Result<Self, ParseError> where R: Read {
		let count = bits.read_bits(9)?;
		let bits_len = bits.read_bits(20)?;

		let mut infos = Vec::with_capacity(count as usize);

		for _ in 0..count {
			infos.push(GameEventInfo::parse(bits)?);
		}

		// TODO: Verify bit length.

		Ok(GameEventList(infos))
	}
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Kind {
	/// This is Local on the server side, but the ID is reused as End when serializing the schema.
	End,
	Str,
	F32,
	I32,
	I16,
	U8,
	Bool,
	Unused
}

impl Kind {
	fn from_id(id: u32) -> Option<Self> {
		Some(match id {
			0 => Kind::End,
			1 => Kind::Str,
			2 => Kind::F32,
			3 => Kind::I32,
			4 => Kind::I16,
			5 => Kind::U8,
			6 => Kind::Bool,
			7 => Kind::Unused,
			_ => return None
		})
	}
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct GameEventInfo {
	pub index: u16,
	pub name: String,
	pub properties: Vec<(Kind, String)>
}

impl GameEventInfo {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Result<Self, ParseError> where R: Read {
		let index = bits.read_bits(9)? as u16;
		let name = bits.read_string()?;
		let mut properties = Vec::new();

		loop {
			let kind = Kind::from_id(bits.read_bits(3)?).unwrap();

			if kind == Kind::End {
				break;
			}

			properties.push((kind, bits.read_string()?));
		}

		Ok(GameEventInfo { index, name, properties })
	}
}