use std::io::{self, Read, Cursor};
use byteorder::{ReadBytesExt, LittleEndian};
use demo::parse::{ParseError, Needed};

pub struct BitReader<R> where R: Read {
	input: R,
	remaining_bytes: usize,
	bits: u32,
	/// Available bits. Must always be above 0.
	available: u8
}

impl<R> BitReader<R> where R: Read {
	pub fn new(input: R, max_bytes: usize) -> io::Result<Self> {
		let mut reader = BitReader {
			input,
			remaining_bytes: max_bytes,
			bits: 0,
			available: 0
		};

		reader.fetch()?;

		Ok(reader)
	}

	pub fn unread_bytes(&self) -> usize {
		self.remaining_bytes
	}

	pub fn has_remaining(&self, needed: usize) -> bool {
		if self.remaining_bytes >= usize::max_value() / 8 {
			true
		} else {
			needed <= self.remaining_bits()
		}
	}

	pub fn has_remaining_bytes(&self, needed: usize) -> bool {
		self.remaining_bytes.saturating_add((self.available / 8) as usize) >= needed
	}

	pub fn remaining_bits(&self) -> usize {
		self.remaining_bytes * 8 + (self.available as usize)
	}

	pub fn remaining_bytes(&self) -> usize {
		self.remaining_bytes + ((self.available / 8) as usize)
	}

	pub fn available_now(&self) -> u8 {
		self.available
	}

	fn fetch(&mut self) -> io::Result<bool> {
		assert_eq!(self.available, 0);

		match self.remaining_bytes {
			0 => return Ok(false),
			1 => {
				self.remaining_bytes = 0;

				self.bits = self.input.read_u8()? as u32;
				self.available = 8;
			},
			2 => {
				self.remaining_bytes = 0;

				self.bits = self.input.read_u16::<LittleEndian>()? as u32;
				self.available = 16;
			},
			3 => {
				self.remaining_bytes = 0;

				self.bits = self.input.read_u16::<LittleEndian>()? as u32;
				self.bits |= (self.input.read_u8()? as u32) << 16;

				self.available = 24;
			},
			_ => {
				self.remaining_bytes -= 4;

				self.bits = self.input.read_u32::<LittleEndian>()? as u32;
				self.available = 32;
			}
		};

		Ok(true)
	}

	pub fn read_bit(&mut self) -> Result<bool, ParseError> {
		if self.available < 1 {
			return Err(ParseError::Needed(Needed::Bits { requested: 1, available: self.available as usize }));
		}

		let bit = (self.bits & 1) == 1;

		self.available -= 1;
		self.bits >>= 1;

		if self.available == 0 {
			self.fetch().map_err(ParseError::Io)?;
		}

		Ok(bit)
	}

	fn read_bits_direct(&mut self, count: u8) -> io::Result<u32> {
		assert!(count > 0);
		assert!(count <= self.available);

		let bits = if count == 32 {
			let bits = self.bits;
			self.bits = 0;
			self.available = 0;

			bits
		} else {
			let bits = self.bits & ((1 << count) - 1);
			self.bits >>= count;
			self.available -= count;

			bits
		};

		if self.available == 0 {
			self.fetch()?;
		}

		Ok(bits)
	}

	pub fn read_bits(&mut self, count: u8) -> Result<u32, ParseError> {
		if count == 0 {
			return Ok(0);
		}

		assert!(count <= 32, "cannot read more than 32 bits from a BitReader at a time.");

		if count <= self.available {
			self.read_bits_direct(count).map_err(ParseError::Io)
		} else {
			let taken = self.available;
			let needed = count - taken;

			if self.remaining_bytes < 4 && (self.remaining_bytes as u8) * 8 < needed {
				return Err(ParseError::Needed(Needed::Bits { requested: count as usize, available: taken as usize + self.remaining_bytes * 8 }));
			}

			let parts = (
				self.read_bits_direct(taken ).map_err(ParseError::Io)?,
				self.read_bits_direct(needed).map_err(ParseError::Io)?
			);

			Ok(parts.0 | (parts.1 << taken))
		}
	}

	pub fn read_u8(&mut self) -> Result<u8, ParseError> {
		self.read_bits(8).map(|x| x as u8)
	}

	pub fn read_u8_array(&mut self, len: usize) -> Result<Vec<u8>, ParseError> {
		let mut data = Vec::with_capacity(len);

		self.read_u8_array_into(&mut data, len)?;

		Ok(data)
	}

	pub fn read_u8_array_into(&mut self, data: &mut Vec<u8>, len: usize) -> Result<(), ParseError> {
		if !self.has_remaining_bytes(len) {
			return Err(ParseError::Needed(Needed::Bytes { requested: len, available: self.remaining_bytes }));
		}

		for _ in 0..len {
			data.push(self.read_u8()?);
		}

		Ok(())
	}

	pub fn read_u16(&mut self) -> Result<u16, ParseError> {
		self.read_bits(16).map(|x| x as u16)
	}

	pub fn read_u32(&mut self) -> Result<u32, ParseError> {
		self.read_bits(32).map(|x| x as u32)
	}

	pub fn read_f32(&mut self) -> Result<f32, ParseError> {
		self.read_u32().map(f32::from_bits)
	}

	pub fn read_i8(&mut self) -> Result<i8, ParseError> {
		self.read_u8().map(|x| x as i8)
	}

	pub fn read_i16(&mut self) -> Result<i16, ParseError> {
		self.read_u16().map(|x| x as i16)
	}

	pub fn read_i32(&mut self) -> Result<i32, ParseError> {
		self.read_u32().map(|x| x as i32)
	}

	pub fn read_var(&mut self) -> Result<u32, ParseError> {
		match self.read_bits(2)? {
			0 => self.read_bits(4),
			1 => self.read_u8().map(u32::from),
			2 => self.read_bits(12),
			_ => self.read_u32()
		}
	}

	pub fn read_coord(&mut self) -> Result<f32, ParseError> {
		// TODO: This is not pure, parse errors are not recoverable.

		let integral = self.read_bit()?;
		let fractional = self.read_bit()?;

		if integral || fractional {
			let sign = self.read_bit()?;

			let integer = if integral {
				self.read_bits(14)? + 1
			} else {
				0
			};

			let fraction = if fractional {
				self.read_bits(5)?
			} else {
				0
			};

			let value = (integer as f32) + (fraction as f32) * 0.03125;

			Ok(if sign { -value } else { value })
		} else {
			Ok(0.0)
		}
	}

	pub fn read_vec3(&mut self) -> Result<(f32, f32, f32), ParseError> {
		// TODO: This is not pure, parse errors are not recoverable.

		let x = self.read_bit()?;
		let y = self.read_bit()?;
		let z = self.read_bit()?;

		Ok((
			if x { self.read_coord()? } else { 0.0 },
			if y { self.read_coord()? } else { 0.0 },
			if z { self.read_coord()? } else { 0.0 }
		))
	}

	pub fn read_string(&mut self) -> Result<String, ParseError> {
		// TODO: This is not pure, parse errors are not recoverable.

		let mut data = Vec::new();

		loop {
			let value = self.read_u8()?;

			if value == 0 {
				break;
			}

			data.push(value);
		}

		String::from_utf8(data).map_err(ParseError::Utf8)
	}

	pub fn read_var_u32(&mut self) -> Result<u32, ParseError> {
		// TODO: This is not pure, parse errors are not recoverable.

		let mut result = 0;

		for index in 0..5 {
			let byte = self.read_u8()?;

			result |= ((byte & 0x7F) as u32) << (7 * index);

			if byte < 128 {
				break;
			}
		}

		Ok(result)
	}

	pub fn end(self) -> (R, u8) {
		(self.input, self.available)
	}
}

#[derive(Debug, Clone)]
pub struct Bits {
	data: Vec<u8>,
	trailing_bits: u8
}

impl Bits {
	pub fn from_bytes(data: Vec<u8>) -> Self {
		Bits { data, trailing_bits: 0 }
	}

	pub fn copy_into<R>(bits: &mut BitReader<R>, count: usize) -> Result<Self, ParseError> where R: Read {
		if !bits.has_remaining(count) {
			return Err(ParseError::Needed(Needed::Bits { requested: count, available: bits.remaining_bits() }));
		}

		let trailing_bits = (count % 8) as u8;
		let bytes = count / 8;

		let mut data = Vec::with_capacity(bytes + if trailing_bits != 0 {1} else {0});
		bits.read_u8_array_into(&mut data, bytes)?;

		if trailing_bits != 0 {
			data.push(bits.read_bits(trailing_bits)? as u8);
		}

		Ok(Bits { data, trailing_bits })
	}

	pub fn reader(&self) -> BitReader<Cursor<&Vec<u8>>> {
		let len = self.data.len();
		let cursor = Cursor::new(&self.data);

		BitReader::new(cursor, len).unwrap()
	}

	pub fn bits_len(&self) -> usize {
		self.data.len() * 8 + (self.trailing_bits as usize)
	}

	pub fn raw_bytes(&self) -> &[u8] {
		&self.data
	}
}