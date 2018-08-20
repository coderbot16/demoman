use demo::bits::BitReader;
use demo::parse::ParseError;
use std::io::Read;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Sound {
	pub entity:      u16, // 11 bits
	pub sound_index: u16, // 13 bits
	pub flags:       Flags, // 9 bits
	pub channel:     Channel, // 3 bits
	pub ambient:     bool,
	pub sentence:    bool
}

impl Default for Sound {
	fn default() -> Self {
		Sound {
			entity: 0,
			sound_index: 0,
			flags: Flags(0),
			channel: Channel::Static,
			ambient: false,
			sentence: false
		}
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SoundDelta {
	pub entity: Option<u16>, // 11 bits
	pub sound_index: Option<u16>, // 13 bits
	pub flags: Option<Flags>, // 9 bits
	pub channel: Option<Channel>,
	pub ambient: bool,
	pub sentence: bool
}

impl SoundDelta {
	pub fn apply(&self, sound: &Sound) -> Sound {
		Sound {
			entity: self.entity.unwrap_or(sound.entity),
			sound_index: self.sound_index.unwrap_or(sound.sound_index),
			flags: self.flags.unwrap_or(sound.flags),
			channel: self.channel.unwrap_or(sound.channel),
			ambient: self.ambient,
			sentence: self.sentence
		}
	}

	pub fn parse<R>(bits: &mut BitReader<R>) -> Result<Self, ParseError> where R: Read {
		Ok(SoundDelta {
			entity: if bits.read_bit()? {
				Some(bits.read_bits(11)? as u16)
			} else {
				None
			},
			sound_index: if bits.read_bit()? {
				Some(bits.read_bits(13)? as u16)
			} else {
				None
			},
			flags: if bits.read_bit()? {
				Some(Flags(bits.read_bits(9)? as u16))
			} else {
				None
			},
			channel: if bits.read_bit()? {
				Some(Channel::from_id(bits.read_bits(3)? as u8))
			} else {
				None
			},
			ambient: bits.read_bit()?,
			sentence: bits.read_bit()?
		})
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SoundData {
	pub sequence: u16,
	pub volume:   u8, // 7 bits, f32 from 0.0 to 1.0 scaled to a 7-bit value
	pub sound_level: u16, // 9 bits
	pub pitch: u8, // 8 bits
	pub delay: i16, // signed, 13 bits
	pub origin: (i16, i16, i16), // 12 bits each, each coordinate is divided by 8.0, signed
	pub speaker: i16 // 12 bits, signed
}

impl Default for SoundData {
	fn default() -> Self {
		SoundData {
			sequence: 0,
			volume: unimplemented!(), //1.0, // TODO
			sound_level: 75, // 75db
			pitch: 100,
			delay: unimplemented!(), // 0.0, // TODO
			origin: (0, 0, 0),
			speaker: -1
		}
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SoundDataDelta {
	pub sequence: SequenceUpdate,
	pub volume: Option<u8>, // 7 bits, f32 from 0.0 to 1.0 scaled to a 7-bit value
	pub sound_level: Option<u16>, // 9 bits
	pub pitch: Option<u8>, // 8 bits
	pub delay: Option<i16>, // signed, 13 bits
	pub origin: (Option<i16>, Option<i16>, Option<i16>), // 12 bits each, each coordinate is divided by 8.0, signed
	pub speaker: Option<i16> // 12 bits, signed
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SequenceUpdate {
	Unchanged,
	Increment,
	Full(u16)
}

impl SequenceUpdate {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Result<Self, ParseError> where R: Read {
		Ok(if bits.read_bit()? {
			SequenceUpdate::Unchanged
		} else {
			if bits.read_bit()? {
				SequenceUpdate::Increment
			} else {
				SequenceUpdate::Full(bits.read_bits(10)? as u16)
			}
		})
	}

	pub fn derive(old: u16, new: u16) -> Self {
		if old == new {
			SequenceUpdate::Unchanged
		} else if old + 1 == new {
			SequenceUpdate::Increment
		} else {
			SequenceUpdate::Full(new)
		}
	}

	pub fn apply(&self, old: u16) -> u16 {
		match *self {
			SequenceUpdate::Unchanged => old,
			SequenceUpdate::Increment => old + 1,
			SequenceUpdate::Full(new) => new
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Channel {
	Auto,
	Weapon,
	Voice,
	Item,
	Body,
	Stream,
	Static,
	/// Reserved as the first voice allocation channel, not used by normal packets.
	Reserved
}

impl Channel {
	fn from_id(id: u8) -> Self {
		match id & 7 {
			0 => Channel::Auto,
			1 => Channel::Weapon,
			2 => Channel::Voice,
			3 => Channel::Item,
			4 => Channel::Body,
			5 => Channel::Stream,
			6 => Channel::Static,
			7 => Channel::Reserved,
			_ => unreachable!()
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Flags(pub u16);