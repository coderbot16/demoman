extern crate dem;
extern crate nom;
extern crate byteorder;
extern crate snap;

use ::nom::IResult;
use dem::demo::header::{self, DemoHeader};
use dem::demo::bits::BitReader;
use dem::demo::usercmd::{UserCmdDelta, PositionUpdate};
use dem::demo::data_table::DataTables;
use dem::packets::{PacketKind, TransferFile, Tick, SignonState, ClassInfo, Decal};
use dem::packets::game_events::GameEventList;
use dem::packets::string_table::{StringTables, CreateStringTable};
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

		let kind = PacketKind::from_id(id as u8).expect("Packet ID cannot be greater than 31");

		print!("  {:>17} | ", format!("{:?}", kind));

		match kind {
			PacketKind::Nop               => println!("Nop"),
			PacketKind::Disconnect        => unimplemented!(),
			PacketKind::TransferFile      => println!("{:?}", TransferFile::parse(&mut bits)),
			PacketKind::Tick              => println!("{:?}", Tick::parse(&mut bits)),
			PacketKind::StringCommand     => println!("{:?}", bits.read_string()),
			PacketKind::SetCvars          => {
				let count = bits.read_u8();
				println!("{} cvars", count);

				for _ in 0..count {
					println!("  {:>17} : {:?} = {:?}", "", bits.read_string().unwrap(), bits.read_string().unwrap());
				}
			},
			PacketKind::SignonState       => println!("{:?}", SignonState::parse(&mut bits)),
			PacketKind::Print             => println!("Print({:?})", bits.read_string()),
			PacketKind::ServerInfo        => println!("{:?}", packets::ServerInfo::parse(&mut bits)),
			PacketKind::DataTable         => unimplemented!(),
			PacketKind::ClassInfo         => println!("{:?}", ClassInfo::parse(&mut bits)),
			PacketKind::Pause             => unimplemented!(),
			PacketKind::CreateStringTable => {
				let create = CreateStringTable::parse(&mut bits);

				println!("Table: {}, Entries: {} / {:?}, Fixed Userdata Size: {:?}", create.name, create.table.strings.len(), create.table.capacity(), create.table.fixed_extra_size());
			},
			PacketKind::UpdateStringTable => {
				// TODO: Broken

				let index_bits = 0;

				let table_id = bits.read_bits(5) as u8;
				let entries = if bits.read_bit() {
					bits.read_u16()
				} else {
					1
				};

				let bits_len = bits.read_bits(20) as u16;

				println!("Table: {}, Entries: {}, Bits: {}", table_id, entries, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			PacketKind::VoiceInit         => {
				// TODO: Changed recently: Medics demo has different format...

				println!("Codec: {:?}, Quality: {}, ???: {}", bits.read_string(), bits.read_u8(), bits.read_u16());
			},
			PacketKind::VoiceData         => {
				let client_sender = bits.read_u8();
				let proximity = bits.read_u8();

				let bits_len = bits.read_u16();

				println!("Sender: {}, Proximity: {}, Bits: {}", client_sender, proximity, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			PacketKind::HltvControl      => unimplemented!(),
			PacketKind::PlaySound        => {
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
			PacketKind::SetEntityView    => println!("Entity: {}", bits.read_bits(11)),
			PacketKind::FixAngle         => {
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

				println!("Relative: {}, (degrees): {:?} [raw: {:?}]", relative, degrees, angles);
			},
			PacketKind::CrosshairAngle   => {
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
			PacketKind::Decal            => println!("{:?}", Decal::parse(&mut bits)),
			PacketKind::TerrainMod       => unimplemented!(),
			PacketKind::UserMessage      => {
				let kind = bits.read_u8();
				let bits_len = bits.read_bits(11) as u16;

				println!("Kind: {}, Bits: {}", kind, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			PacketKind::EntityMessage    => {
				let entity = bits.read_bits(11) as u16;
				let class = bits.read_bits(9) as u16;

				let bits_len = bits.read_bits(11) as u16;

				println!("Entity: {}, Class: {}, Bits: {}", entity, class, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			PacketKind::GameEvent        => {
				let bits_len = bits.read_bits(11);

				if bits_len < 9 {
					println!("Error: Too small! Bits: {}", bits_len);
					break;
				}

				let id = bits.read_bits(9);

				println!("Event ID: {}, Bits: {}", id, bits_len);

				for _ in 0..bits_len-9 {
					bits.read_bit();
				}
			},
			PacketKind::Entities         => {
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

				println!("Entries: {} updated / {} max, Baseline: {} Update Baseline: {}, Delta From Tick: {:?}, Bits: {}", updated, max_entries, baseline, update_baseline, delta_from_tick, bits_len);

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
			PacketKind::TempEntities     => {
				let count = bits.read_u8();
				let bits_len = bits.read_var_u32();

				println!("Count: {}, Bits: {}", count, bits_len);

				for _ in 0..bits_len {
					bits.read_bit();
				}
			},
			PacketKind::Prefetch         => {
				println!("???: {}, ID: {}", bits.read_bit(), bits.read_bits(13));
			},
			PacketKind::PluginMenu       => {
				// TODO: BROKEN

				let kind = bits.read_u16();
				let len = bits.read_u16();

				println!("Kind: {}, len: {}", kind, len);

				for _ in 0..len {
					print!("{} ", bits.read_u8());
				}

				println!("  Don't know how to handle a Menu!");
				break;
			},
			PacketKind::GameEventList    => {
				let event_list = GameEventList::parse(&mut bits).0;

				println!("{} events not shown", event_list.len());
			},
			PacketKind::GetCvar          => {
				// TODO: BROKEN?

				println!("Cookie: {}, CVar: {:?}", bits.read_u32(), bits.read_string());

				println!("  Don't know how to handle a GetCvarValue!");
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