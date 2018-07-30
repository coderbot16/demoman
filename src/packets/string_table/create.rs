use demo::bits::BitReader;
use std::io::{Read, Cursor};
use packets::string_table::StringTable;

extern crate snap;

#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum CompressionType {
	/// Google Snappy compression
	Snappy = 0x534E4150,
	/// LZSS (variant of LZ77) based compression
	Lzss   = 0x4C5A5353
}

impl CompressionType {
	fn from_id(id: u32) -> Result<Self, u32> {
		if id == CompressionType::Snappy as u32 {
			Ok(CompressionType::Snappy)
		} else if id == CompressionType::Lzss as u32 {
			Ok(CompressionType::Lzss)
		} else {
			Err(id)
		}
	}
}

pub struct CreateStringTable {
	pub name:  String,
	pub table: StringTable
}

impl CreateStringTable {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		let name = bits.read_string().unwrap();
		let max_entries = bits.read_u16();

		assert_ne!(max_entries, 0);

		let index_bits = (16 - max_entries.leading_zeros()) as u8 - 1;
		let entries = bits.read_bits(index_bits + 1) as u16;
		let bits_len = bits.read_var_u32();

		// Size and Bits Size
		let fixed_userdata_size = if bits.read_bit()  {
			Some((bits.read_bits(12) as u16, bits.read_bits(4) as u8))
		} else {
			None
		};

		let mut table = StringTable::create(entries as usize, max_entries as usize, fixed_userdata_size.map(|(bytes, bits)| bits));

		let start_rem_bits = bits.remaining_bits();

		if bits.read_bit() {
			let uncompressed_size = bits.read_u32();
			let compressed_size = bits.read_u32();

			assert!(compressed_size > 4);

			let compressed_size = compressed_size - 4;

			let compression = match CompressionType::from_id(bits.read_u32().swap_bytes()) {
				Ok(compression) => compression,
				Err(id) => panic!("Unexpected String Table compression magic: expected 0x534E4150 ('SNAP') or 0x4C5A5353 ('LZSS'), got 0x{:08X}", id)
			};

			//println!("  Using snappy | uncompressed bytes: {}, compressed bytes: {}", uncompressed_size, compressed_size);

			let mut compressed = Vec::with_capacity(compressed_size as usize);
			for _ in 0..compressed_size {
				compressed.push(bits.read_u8());
			}

			let uncompressed = match compression {
				CompressionType::Snappy => {
					let mut snappy = snap::Decoder::new();
					snappy.decompress_vec(&compressed).expect("invalid snappy data")
				},
				CompressionType::Lzss => {
					println!("ERROR!");
					println!("LZSS: Uncompressed bytes: {}, Compressed bytes: {}", uncompressed_size, compressed_size);
					println!("Compression type {:?} is unsupported! Returning empty table!", compression);

					println!("LZSS Format Dump:");
					for byte in compressed {
						print!("{:02X} ", byte);
					}
					println!();

					return CreateStringTable {
						name,
						table
					};
				}
			};

			let mut cursor = Cursor::new(&uncompressed);
			let mut bits = BitReader::new(&mut cursor, uncompressed_size as usize);

			table.update(&mut bits, entries);
		} else {
			table.update(bits, entries);
		};

		// +1 accounts for the compression/no compression bit that is NOT counted in the bits len normally.
		assert_eq!(start_rem_bits - bits.remaining_bits(), (bits_len) as usize + 1, "Unexpected amount of bits read!");

		CreateStringTable {
			name,
			table
		}
	}
}