use demo::bits::BitReader;
use std::io::{Read, Cursor};
use packets::string_table::StringTable;
use packets::CreateStringTable;

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

pub struct NewStringTable {
	pub name:  String,
	pub table: StringTable
}

impl NewStringTable {
	pub fn from_packet(packet: CreateStringTable) -> Self {
		let mut table = StringTable::create(packet.entries as usize, packet.max_entries as usize, packet.fixed_userdata_size.map(|(bytes, bits)| bits));

		let mut bits = packet.data.reader();

		if packet.compressed {
			let uncompressed_size = bits.read_u32();
			let compressed_size = bits.read_u32();

			assert!(compressed_size > 4);

			let compressed_size = compressed_size - 4;

			let compression = match CompressionType::from_id(bits.read_u32().swap_bytes()) {
				Ok(compression) => compression,
				Err(id) => panic!("Unexpected String Table compression magic: expected 0x534E4150 ('SNAP') or 0x4C5A5353 ('LZSS'), got 0x{:08X}", id)
			};

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

					return NewStringTable {
						name: packet.name,
						table
					};
				}
			};

			let mut cursor = Cursor::new(&uncompressed);
			let mut bits = BitReader::new(&mut cursor, uncompressed_size as usize);

			table.update(&mut bits, packet.entries);
		} else {
			table.update(&mut bits, packet.entries);
		};

		NewStringTable {
			name: packet.name,
			table
		}
	}
}