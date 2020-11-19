use std::collections::VecDeque;
use crate::demo::bits::BitReader;
use crate::demo::parse::ParseError;

mod create;

pub use self::create::NewStringTable;

#[derive(Debug, Clone)]
pub struct StringTables(pub Vec<(String, StringTablePair)>);

impl StringTables {
	pub fn parse(bits: &mut BitReader) -> Result<Self, ParseError> {
		let count = bits.read_u8()?;
		let mut tables = Vec::with_capacity(count as usize);
		
		for _ in 0..count {
			let name = bits.read_string()?;
			let table = StringTablePair::parse(bits)?;

			tables.push((name, table));
		}

		Ok(StringTables(tables))
	}
}

#[derive(Debug, Clone)]
pub struct StringTablePair {
	/// Primary string table.
	pub primary: StringTable,
	/// A secondary string table.
	// TODO: What is the purpose compared to the primary one?
	pub client: Option<StringTable>
}

impl StringTablePair {
	pub fn parse(bits: &mut BitReader) -> Result<Self, ParseError>{
		Ok(StringTablePair {
			primary: StringTable::parse(bits)?,
			client: if bits.read_bit()? { Some(StringTable::parse(bits)?) } else { None }
		})
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Extra {
	Bits { count: u8, data: u16 },
	Bytes(Vec<u8>),
	None
}

#[derive(Debug, Clone)]
pub struct StringTable {
	pub strings: Vec<(String, Extra)>,
	capacity: Option<usize>,
	fixed_extra_size: Option<u8>
}

impl StringTable {
	// TODO: make private
	pub fn create(entries: usize, capacity: usize, fixed_extra_size: Option<u8>) -> Self {
		assert!(entries <= capacity);

		let mut strings = Vec::with_capacity(capacity);
		for _ in 0..entries {
			strings.push((String::new(), Extra::None));
		}

		StringTable {
			strings,
			capacity: Some(capacity),
			fixed_extra_size
		}
	}

	pub fn fixed_extra_size(&self) -> Option<u8> {
		self.fixed_extra_size
	}

	pub fn capacity(&self) -> Option<usize> {
		self.capacity
	}

	pub fn parse(bits: &mut BitReader) -> Result<Self, ParseError> {
		let count = bits.read_u16()?;
		let mut strings = Vec::with_capacity(usize::from(count));

		for _ in 0..count {
			let string = bits.read_string()?;

			let data = if bits.read_bit()? {
				let data_size = usize::from(bits.read_u16()?);

				Extra::Bytes(bits.read_u8_array(data_size)?)
			} else {
				Extra::None
			};

			strings.push((string, data));
		}

		Ok(StringTable {
			strings,
			capacity: None,
			fixed_extra_size: None
		})
	}

	pub fn update(&mut self, bits: &mut BitReader, updated: u16) -> Result<(), ParseError> {
		let index_bits = (16 - (self.capacity.unwrap() as u16).leading_zeros()) as u8 - 1;

		let mut tracker = StateTracker::new();
		let max_index = (self.strings.len() - 1) as u32;

		for _ in 0..updated {
			let index = if bits.read_bit()? { None } else { Some(bits.read_bits(index_bits)?) };

			let partial;
			let string;

			if bits.read_bit()? {
				partial = if bits.read_bit()? {
					Some(Partial {
						history_index: bits.read_bits(5)? as u8,
						matching: bits.read_bits(5)? as u8
					})
				} else {
					None
				};

				string = Some(bits.read_string()?);
			} else {
				partial = None;
				string = None;
			}

			let row = CompressedRow {
				index,
				partial,
				string
			};

			let (index, string) = tracker.read(row).map_err(|invalid_index|
				ParseError::OutOfBounds { name: "string_table::HistoryIndex", value: u32::from(invalid_index.index), min: 0, max: u32::from(invalid_index.len - 1)}
			)?;

			let extra = if bits.read_bit()? {
				match self.fixed_extra_size {
					Some(bits_len) => {
						let data = bits.read_bits(bits_len)?;

						Extra::Bits { count: bits_len, data: data as u16 }
					}
					None => {
						let bytes = bits.read_bits(14)? as u16;
						let data = bits.read_u8_array(bytes as usize)?;

						Extra::Bytes(data)
					}
				}
			} else {
				Extra::None
			};

			match self.strings.get_mut(index as usize) {
				Some(row) => {
					if let Some(string) = string {
						row.0 = string;
					}

					if extra != Extra::None {
						row.1 = extra;
					}
				},
				None => return Err(ParseError::OutOfBounds { name: "string_table::StringIndex", value: index, min: 0, max: max_index})
			}
		}

		Ok(())
	}
}

// -- Wire format --

/// Reference to an invalid out of bounds history index.
#[derive(Debug, Clone)]
struct InvalidHistoryIndex {
	index: u8,
	len: u8
}

struct Partial {
	history_index: u8,
	matching: u8
}

struct CompressedRow {
	index:     Option<u32>,
	partial:   Option<Partial>,
	string:    Option<String>,
}

struct StateTracker {
	predicted_index: u32,
	history: VecDeque<String>
}

impl StateTracker {
	fn new() -> Self {
		StateTracker {
			predicted_index: 0,
			history: VecDeque::with_capacity(32)
		}
	}

	fn read(&mut self, row: CompressedRow) -> Result<(u32, Option<String>), InvalidHistoryIndex> {
		let index = row.index.unwrap_or(self.predicted_index);
		self.predicted_index = index + 1;

		let string = match row.string {
			Some(string) => string,
			None => return Ok((index, None))
		};

		let string = match row.partial {
			Some(Partial { history_index, matching }) => {
				let partial = match self.history.get(history_index as usize) {
					Some(history) => history.split_at(matching as usize).0,
					None => return Err(InvalidHistoryIndex { index: history_index, len: self.history.len() as u8 })
				};

				let mut full = partial.to_string();
				full.push_str(&string);

				full
			},
			None => string
		};

		while self.history.len() >= 32 {
			self.history.pop_front();
		}

		self.history.push_back(string.clone());

		Ok((index, Some(string)))
	}
}

