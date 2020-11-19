pub struct Reader<'a> {
	bytes: &'a [u8]
}

impl<'a> Reader<'a> {
	pub fn new(bytes: &'a [u8]) -> Self {
		Reader {
			bytes
		}
	}

	pub fn bytes(&mut self, len: usize) -> &'a [u8] {
		let (requested, rest) = self.bytes.split_at(len);
		self.bytes = rest;

		requested
	}

	pub fn u32(&mut self) -> u32 {
		if let &[a, b, c, d] = self.bytes(4) {
			u32::from_le_bytes([a, b, c, d])
		} else {
			unreachable!()
		}
	}

	pub fn i32(&mut self) -> i32 {
		self.u32() as i32
	}

	pub fn f32(&mut self) -> f32 {
		f32::from_bits(self.u32())
	}
}
