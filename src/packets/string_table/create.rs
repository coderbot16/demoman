use demo::bits::BitReader;
use demo::parse::ParseError;
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
	pub fn from_packet(packet: CreateStringTable) -> Result<Self, ParseError> {
		let mut table = StringTable::create(packet.entries as usize, packet.max_entries as usize, packet.fixed_userdata_size.map(|(bytes, bits)| bits));

		let mut bits = packet.data.reader();

		if packet.compressed {
			let uncompressed_size = bits.read_u32()?;
			let compressed_size = bits.read_u32()?;

			if compressed_size < 4 {
				return Err(ParseError::Custom("string table compressed size is too small, must be at least 4 to contain compression magic".into()));
			}

			let compressed_size = compressed_size - 4;

			let compression = match CompressionType::from_id(bits.read_u32()?.swap_bytes()) {
				Ok(compression) => compression,
				Err(id) => return Err(ParseError::BadEnumIndex { name: "string_table::CompressionType", value: id })
			};

			let compressed = bits.read_u8_array(compressed_size as usize)?;

			let uncompressed = match compression {
				CompressionType::Snappy => {
					let mut snappy = snap::Decoder::new();
					snappy.decompress_vec(&compressed).expect("invalid snappy data")
				},
				CompressionType::Lzss => {
					println!("ERROR!");
					println!("LZSS: Uncompressed bytes: {}, Compressed bytes: {}", uncompressed_size, compressed_size);
					println!("Compression type LZSS is unsupported! Returning empty table!");

					println!("LZSS Format Dump:");
					for byte in compressed {
						print!("{:02X} ", byte);
					}
					println!();

					return Ok(NewStringTable {
						name: packet.name,
						table
					});
				}
			};

			let mut bits = BitReader::new(&uncompressed);

			table.update(&mut bits, packet.entries);
		} else {
			table.update(&mut bits, packet.entries);
		};

		Ok(NewStringTable {
			name: packet.name,
			table
		})
	}
}