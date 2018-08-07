use demo::bits::BitReader;
use std::io::Read;

pub struct SoundDelta {
	pub entity: Option<u16>, // 11 bits
	pub sound_index: Option<u16>, // 13 bits
	pub flags: Option<u16>, // 9 bits
	pub channel: Option<u8>, // 3 bits
	pub ambient: bool,
	pub sentence: bool,
	pub data: Option<SoundData>
}

pub struct SoundData {
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
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		if bits.read_bit() {
			SequenceUpdate::Unchanged
		} else {
			if bits.read_bit() {
				SequenceUpdate::Increment
			} else {
				SequenceUpdate::Full(bits.read_bits(10))
			}
		}
	}

	pub fn determine(old: u16, new: u16) -> Self {
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
			SequenceUpdate::PlusOne   => old + 1,
			SequenceUpdate::Full(new) => new
		}
	}
}