extern crate dem;
extern crate nom;
extern crate byteorder;
extern crate snap;

use ::nom::IResult;
use dem::demo::header::{self, DemoHeader};
use dem::demo::bits::BitReader;
use dem::demo::usercmd::{UserCmdDelta, PositionUpdate};
use dem::demo::data_table::DataTables;
use dem::packets::{TransferFile, Tick, SignonState, ClassInfo, Decal};
use dem::packets::game_events::GameEventList;
use dem::packets::string_table::StringTables;
use dem::demo::frame::{Frame, FramePayload};
use dem::packets::string_table::StringTable;
use dem::packets;

use std::io::{BufReader, Read, Seek, SeekFrom, Cursor};
use std::fs::File;
use byteorder::{ReadBytesExt, LittleEndian};

//const PATH: &str = "/home/coderbot/Source/HowToMedicFortress_coderbot_1200_USA.dem";
const PATH: &str = "/home/coderbot/.steam/steam/steamapps/common/Team Fortress 2/tf/demos/2017-12-23_16-43-13.dem";

fn main() {
	let mut file = BufReader::new(File::open(PATH).unwrap());

	let mut buf = [0; header::HEADER_LENGTH];
	file.read(&mut buf[0..]).unwrap();
	
	let demo = DemoHeader::parse(&mut buf[0..]).unwrap().1;
	println!("Demo protocol {}, carrying network protocol {}", demo.demo_protocol, demo.network_protocol);
	println!("Server: {:?}", demo.server_name);
	println!("Client: {:?}", demo.client_name);
	println!("Map: {:?}", demo.map_name);
	println!("Game directory: {:?}", demo.game_directory);
	println!("Time: {} seconds, {} ticks, {} frames", demo.playback_seconds, demo.ticks, demo.frames);

	let signon_end = header::HEADER_LENGTH as u64 + demo.signon_length as u64;

	println!();
	println!("-- START OF SIGNON DATA ({} bytes) --", demo.signon_length);
	println!();
	
	// Iterate over a limited amount of packets
	for _ in 0..4096 {
		let offset = file.seek(SeekFrom::Current(0)).unwrap();

		if offset == signon_end {
			println!();
			println!("-- END OF SIGNON DATA --");
			println!();
		}

		let frame = Frame::parse(&mut file);
		print!("T: {} ", frame.tick);

		match frame.payload {
			FramePayload::SignonUpdate(update) | FramePayload::Update(update) => {
				println!("| Update ({} packet bytes)", update.packets.len());

				parse_update(update.packets, &demo);
			},
			FramePayload::TickSync => println!("| Tick Sync"),
			FramePayload::ConsoleCommand(command) => println!("> {}", command),
			FramePayload::UserCmdDelta(delta) => println!("| UserCmdDelta (hidden)"),
			FramePayload::DataTables(tables) => println!("| Data Tables - {} tables, {} class links", tables.tables.len(), tables.links.len()),
			FramePayload::Stop => {
				println!("| Stop");
				break;
			},
			FramePayload::StringTables(tables) => {
				println!("| String Tables - {} tables", tables.0.len());

				for (index, &(ref name, ref pair)) in tables.0.iter().enumerate() {
					print!("  #{} | {}: {} primary strings, ", index, name, pair.primary.strings.len());
					match &pair.client {
						&Some(ref table) => println!("{} client strings", table.strings.len()),
						&None => println!("no client strings")
					}

					/*for (index, &(ref string, ref extra)) in table.1.primary.iter().enumerate() {
						print!("    #{}: {} ", index, string);
						match extra {
							&Some(ref bytes) => println!("= {:?}", bytes),
							&None => println!()
						}
					}*/
				}
			}
		}
	}
}

/*println!("Data Tables Markdown Dump:");
for table in &tables.tables {
	println!("### {} [{}]", table.name, if table.needs_decoder { "Decoder Needed" } else { "Decoder Not Needed" });
	println!();

	println!("| Field | Flags | Data |");
	println!("|-------|-------|------|");

	for row in &table.rows {
		println!("|{}|{}|{:?}|", row.name, row.flags, row.data);
	}

	println!();
}*/

fn parse_update(mut data: Vec<u8>, demo: &DemoHeader) {
	/*for alignment in 0..8 {
		let len = data.len();
		let mut cursor = Cursor::new(&mut data);
		let mut bits = BitReader::new(&mut cursor, len);

		let mut file = File::create(format!("align-{}", alignment)).unwrap();

		bits.read_bits(alignment);

		while bits.remaining_bits() >= 8 {
			use byteorder::WriteBytesExt;;

			file.write_u8(bits.read_u8()).unwrap();
		}
	}

	::std::process::exit(0);*/

	let len = data.len();
	let mut cursor = Cursor::new(&mut data);
	let mut bits = BitReader::new(&mut cursor, len);

	assert!(demo.network_protocol > 10, "Network protocols less than 10 do not have fixed_time and fixed_time_stdev in Tick, this is not handled yet!");

	while bits.remaining_bits() >= 6 {
		let id = bits.read_bits(6);
		print!("  Packet ID: {} | ", id);

		match id {
			0 => println!("Nop"),
			2 => println!("{:?}", TransferFile::parse(&mut bits)),
			3 => println!("{:?}", Tick::parse(&mut bits)),
			4 => println!("StringCommand({:?})", bits.read_string()),
			5 => {
				let count = bits.read_u8();
				println!("SetCvars | {} cvars", count);

				for _ in 0..count {
					println!("    {:?} = {:?}", bits.read_string().unwrap(), bits.read_string().unwrap());
				}
			},
			6 => println!("{:?}", SignonState::parse(&mut bits)),
			7 => println!("Print({:?})", bits.read_string()),
			8 => println!("{:?}", packets::ServerInfo::parse(&mut bits)),
			10 => println!("{:?}", ClassInfo::parse(&mut bits)),
			12 => {
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

				println!("CreateStringTable | Table: {}, Entries: {} / {} ({} index bits), Fixed Userdata Size: {:?}, Bits: {}", name, entries, max_entries, index_bits, fixed_userdata_size, bits_len);

				let mut table = StringTable::create(entries as usize, max_entries as usize, fixed_userdata_size.map(|(bytes, bits)| bits));

				let start_rem_bits = bits.remaining_bits();

				if bits.read_bit() {
					let uncompressed_size = bits.read_u32();
					let compressed_size = bits.read_u32();

					assert!(compressed_size > 4);

					let compressed_size = compressed_size - 4;
					let magic = bits.read_u32().swap_bytes();

					// 'SNAP' in big-endian
					const SNAP: u32 = 0x534E4150;

					assert_eq!(magic, SNAP, "Unexpected String Table compression magic: expected 0x534E4150 ('SNAP')");

					//println!("  Using snappy | uncompressed bytes: {}, compressed bytes: {}", uncompressed_size, compressed_size);

					let mut compressed = Vec::with_capacity(compressed_size as usize);
					for _ in 0..compressed_size {
						compressed.push(bits.read_u8());
					}

					let mut snappy = snap::Decoder::new();
					let uncompressed = snappy.decompress_vec(&compressed).expect("invalid snappy data");

					let mut cursor = Cursor::new(&uncompressed);
					let mut bits = BitReader::new(&mut cursor, uncompressed_size as usize);

					table.update(&mut bits, entries);
				} else {
					table.update(&mut bits, entries);
				};

				/*use dem::packets::string_table::Extra;

				for (index, &(ref string, ref extra)) in table.strings.iter().enumerate() {
					print!("    #{}: {} ", index, string);
					match extra {
						&Extra::Bytes(ref bytes) => println!("= {:?}", bytes),
						&Extra::Bits { count, data } => println!("= {}", data),
						&Extra::None => println!()
					}
				}*/

				// +1 accounts for the compression/no compression bit that is NOT counted in the bits len normally.
				assert_eq!(start_rem_bits - bits.remaining_bits(), (bits_len) as usize + 1, "Unexpected amount of bits read!");

				continue;
			},
			13 => {
				// TODO: Broken

				let index_bits = 0;

				let table_id = bits.read_bits(5) as u8;
				let entries = if bits.read_bit() {
					bits.read_u16()
				} else {
					1
				};

				let bits_len = bits.read_bits(20) as u16;

				println!("UpdateStringTable | Table: {}, Entries: {}, Bits: {}", table_id, entries, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			14 => {
				// TODO: Changed recently: Medics demo has different format...

				println!("VoiceInit | Codec: {:?}, Quality: {}, ???: {}", bits.read_string(), bits.read_u8(), bits.read_u16());
			},
			15 => {
				let client_sender = bits.read_u8();
				let proximity = bits.read_u8();

				let bits_len = bits.read_u16();

				println!("VoiceData | Sender: {}, Proximity: {}, Bits: {}", client_sender, proximity, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			17 => {
				print!("Sound | ");

				let reliable = bits.read_bit();

				if reliable {
					let bit_len = bits.read_u8();

					for _ in 0..bit_len {
						bits.read_bit();
					}

					println!("Reliable: {} bits", bit_len);
				} else {
					let sounds = bits.read_u8();
					let bit_len = bits.read_u16();

					for _ in 0..bit_len {
						bits.read_bit();
					}

					println!("Unreliable: {} sounds, {} bits", sounds, bit_len);
				};
			},
			18 => println!("SetEntityView({})", bits.read_bits(11)),
			19 => {
				// TODO: BROKEN

				let relative = bits.read_bit();

				let angles = (
					bits.read_u16(),
					bits.read_u16(),
					bits.read_u16()
				);

				let degrees = (
					(angles.0 as f32) * 360.0 / 65536.0,
					(angles.1 as f32) * 360.0 / 65536.0,
					(angles.2 as f32) * 360.0 / 65536.0
				);

				println!("FixAngles | Relative: {}, (degrees): {:?} [raw: {:?}]", relative, degrees, angles);
			},
			20 => {
				// TODO: BROKEN

				let angles = (
					bits.read_u16(),
					bits.read_u16(),
					bits.read_u16()
				);

				let degrees = (
					(angles.0 as f32) * 360.0 / 65536.0,
					(angles.1 as f32) * 360.0 / 65536.0,
					(angles.2 as f32) * 360.0 / 65536.0
				);

				println!("Angles (degrees): {:?} [raw: {:?}]", degrees, angles);
			},
			21 => println!("{:?}", Decal::parse(&mut bits)),
			23 => {
				let kind = bits.read_u8();
				let bits_len = bits.read_bits(11) as u16;

				println!("UserMessage | Kind: {}, Bits: {}", kind, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			24 => {
				let entity = bits.read_bits(11) as u16;
				let class = bits.read_bits(9) as u16;

				let bits_len = bits.read_bits(11) as u16;

				println!("EntityMessage | Entity: {}, Class: {}, Bits: {}", entity, class, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			25 => {
				let bits_len = bits.read_bits(11);

				if bits_len < 9 {
					println!("GameEvent | Error: Too small! Bits: {}", bits_len);
					break;
				}

				let id = bits.read_bits(9);

				println!("GameEvent | Event ID: {}, Bits: {}", id, bits_len);

				for _ in 0..bits_len-9 {
					bits.read_bit();
				}
			},
			26 => {
				let max_entries = bits.read_bits(11) as u16;

				let delta_from_tick = if bits.read_bit() {
					Some(bits.read_u32())
				} else {
					None
				};

				let baseline = bits.read_bit();
				let updated = bits.read_bits(11);
				let bits_len = bits.read_bits(20);
				let update_baseline = bits.read_bit();

				println!("Entities | Entries: {} updated / {} max, Baseline: {} Update Baseline: {}, Delta From Tick: {:?}, Bits: {}", updated, max_entries, baseline, update_baseline, delta_from_tick, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}

				/*#[derive(Debug, Eq, PartialEq)]
				enum UpdateType {
					EnterPvs,
					LeavePvs,
					Delete,
					Delta,
					Finished,
					Preserve
				}

				let mut remaining_headers = updated;
				loop {
					remaining_headers -= 1;
					let is_entity = remaining_headers >= 0;

					let base_update_type = if is_entity {
						Some(match (bits.read_bit(), bits.read_bit()) {
							(false, false) => UpdateType::Delta,
							(false, true) => UpdateType::EnterPvs,
							(true, false) => UpdateType::LeavePvs,
							(true, true) => UpdateType::Delete
						})
					} else {
						None
					};

					loop {
						// TODO: UpdateType::Finish when all entities have been updated.

						let update_type = base_update_type.unwrap_or(UpdateType::Preserve);

						match update_type {
							UpdateType::EnterPvs => (),
							UpdateType::LeavePvs | UpdateType::Delete => (),
							UpdateType::Delta => (),
							UpdateType::Preserve => (),
							UpdateType::Finished => unreachable!()
						}

						if update_type != UpdateType::Preserve {
							break;
						}
					}
				}

				println!("    Update Type: {:?}", update_type);*/

				//break;
			},
			27 => {
				let count = bits.read_u8();
				let bits_len = bits.read_var_u32();

				println!("TempEntities | Count: {}, Bits: {}", count, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			28 => {
				println!("Prefetch | ???: {}, ID: {}", bits.read_bit(), bits.read_bits(13));
			},
			29 => {
				// TODO: BROKEN

				let kind = bits.read_u16();
				let len = bits.read_u16();

				println!("Menu kind: {}, len: {}", kind, len);

				for _ in 0..len {
					print!("{} ", bits.read_u8());
				}

				println!("  Don't know how to handle a Menu!");
				break;
			},
			30 => {
				let event_list = GameEventList::parse(&mut bits).0;

				println!("GameEventList | {} events not shown", event_list.len());
			},
			31 => {
				// TODO: BROKEN?

				println!("GetCVarValue: cookie: {}, cvar: {:?}", bits.read_u32(), bits.read_string());

				println!("  Don't know how to handle a GetCvarValue!");
				break;
			},
			_ => {
				println!("Unknown");
				break;
			}
		}
	}

	println!("  Trailing Bits: {}", bits.remaining_bits());

	if bits.remaining_bits() >= 6 {
		println!(" === SOME PACKETS NOT PARSED ==");
		::std::process::exit(0);
	}
}