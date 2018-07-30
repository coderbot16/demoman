use std::io::{self, Read, Cursor};
use byteorder::{ReadBytesExt, LittleEndian};
use std::string::FromUtf8Error;

pub struct BitReader<R> where R: Read {
	input: R,
	remaining_bytes: usize,
	bits: u32,
	/// Available bits. Must always be above 0.
	available: u8
}

impl<R> BitReader<R> where R: Read {
	pub fn new(input: R, max_bytes: usize) -> Self {
		let mut reader = BitReader {
			input,
			remaining_bytes: max_bytes,
			bits: 0,
			available: 0
		};

		// TODO: Error Handling?
		reader.fetch().expect("Failed to fetch bytes!");

		reader
	}

	pub fn unread_bytes(&self) -> usize {
		self.remaining_bytes
	}

	pub fn remaining_bits(&self) -> usize {
		self.remaining_bytes * 8 + (self.available as usize)
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

	pub fn read_bit(&mut self) -> bool {
		assert!(self.available > 0);

		let bit = (self.bits & 1) == 1;

		self.available -= 1;
		self.bits >>= 1;

		if self.available == 0 {
			// TODO: Error Handling
			self.fetch().expect("Failed to fetch bytes!");
		}

		bit
	}

	fn read_bits_direct(&mut self, count: u8, require: bool) -> u32 {
		assert!(count <= self.available);
		assert!(self.available > 0);

		let bits = self.bits & if count < 32 { ((1 << count) - 1) } else { u32::max_value() };

		self.available -= count;

		if count == 32 {
			self.bits = 0;
		} else {
			self.bits >>= count;
		}

		if self.available == 0 {
			// TODO: Error Handling
			if !self.fetch().expect("Failed to fetch bytes!") && require {
				panic!("No more bytes!");
			}
		}

		bits
	}

	pub fn read_bits(&mut self, count: u8) -> u32 {
		assert!(count <= 32, "cannot read more than 32 bits from a BitReader at a time.");

		if count <= self.available {
			self.read_bits_direct(count, false)
		} else {
			let taken = self.available;
			let needed = count - taken;

			self.read_bits_direct(taken, true) | (self.read_bits_direct(needed, false) << taken)
		}
	}

	pub fn read_u8(&mut self) -> u8 {
		self.read_bits(8) as u8
	}

	pub fn read_u8_array(&mut self, len: usize) -> Vec<u8> {
		let mut data = Vec::with_capacity(len);

		for _ in 0..len {
			data.push(self.read_u8());
		}

		data
	}

	pub fn read_u16(&mut self) -> u16 {
		self.read_bits(16) as u16
	}

	pub fn read_u32(&mut self) -> u32 {
		self.read_bits(32)
	}

	pub fn read_f32(&mut self) -> f32 {
		f32::from_bits(self.read_u32())
	}

	pub fn read_i8(&mut self) -> i8 {
		self.read_u8() as i8
	}

	pub fn read_i16(&mut self) -> i16 {
		self.read_u16() as i16
	}

	pub fn read_i32(&mut self) -> i32 {
		self.read_u16() as i32
	}

	pub fn read_var(&mut self) -> u32 {
		match self.read_bits(2) {
			0 => self.read_bits(4) as u32,
			1 => self.read_u8() as u32,
			2 => self.read_bits(12) as u32,
			_ => self.read_u32()
		}
	}

	pub fn read_coord(&mut self) -> f32 {
		let integral = self.read_bit();
		let fractional = self.read_bit();

		if integral || fractional {
			let sign = self.read_bit();

			let integer = if integral {
				self.read_bits(14) + 1
			} else {
				0
			};

			let fraction = if fractional {
				self.read_bits(5)
			} else {
				0
			};

			let value = (integer as f32) + (fraction as f32) * 0.03125;

			if sign { -value } else { value }
		} else {
			0.0
		}
	}

	pub fn read_vec3(&mut self) -> (f32, f32, f32) {
		let x = self.read_bit();
		let y = self.read_bit();
		let z = self.read_bit();

		(
			if x { self.read_coord() } else { 0.0 },
			if y { self.read_coord() } else { 0.0 },
			if z { self.read_coord() } else { 0.0 }
		)
	}

	pub fn read_string(&mut self) -> Result<String, FromUtf8Error> {
		let mut data = Vec::new();

		loop {
			let value = self.read_u8();

			if value == 0 {
				break;
			}

			data.push(value);
		}

		String::from_utf8(data)
	}

	pub fn read_var_u32(&mut self) -> u32 {
		let mut result = 0;

		for index in 0..5 {
			let byte = self.read_u8();

			result |= ((byte & 0x7F) as u32) << (7 * index);

			if byte < 128 {
				break;
			}
		}

		result
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

	pub fn copy_into<R>(bits: &mut BitReader<R>, count: usize) -> Self where R: Read {
		let trailing_bits = (count % 8) as u8;
		let bytes = count / 8;

		let mut data = Vec::with_capacity(bytes + if trailing_bits != 0 {1} else {0});

		for _ in 0..bytes {
			data.push(bits.read_u8());
		}

		if trailing_bits != 0 {
			data.push(bits.read_bits(trailing_bits) as u8);
		}

		Bits { data, trailing_bits }
	}

	pub fn reader(&self) -> BitReader<Cursor<&Vec<u8>>> {
		let len = self.data.len();
		let cursor = Cursor::new(&self.data);

		BitReader::new(cursor, len)
	}

	pub fn bits_len(&self) -> usize {
		self.data.len() * 8 + (self.trailing_bits as usize)
	}
}