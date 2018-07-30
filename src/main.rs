extern crate dem;
extern crate nom;
extern crate byteorder;
extern crate snap;

use dem::demo::header::{self, DemoHeader};
use dem::demo::bits::{BitReader, Bits};
use dem::demo::usercmd::{UserCmdDelta, PositionUpdate};
use dem::demo::data_table::DataTables;
use dem::packets::{PacketKind, TransferFile, Tick, SetCvars, SignonState, ClassInfo, Decal, VoiceInit, Prefetch, VoiceData, PlaySound, UserMessage, EntityMessage, GameEvent, UpdateStringTable, Entities, TempEntities, FixAngle};
use dem::packets::game_events::GameEventList;
use dem::packets::string_table::{StringTables, CreateStringTable};
use dem::demo::frame::{Frame, FramePayload};
use dem::packets::string_table::StringTable;
use dem::packets;

use std::io::{BufReader, Read, Seek, SeekFrom};
use std::fs::File;
use byteorder::{ReadBytesExt, LittleEndian};

//const PATH: &str = "/home/coderbot/Source/HowToMedicFortress_coderbot_1200_USA.dem";
//const PATH: &str = "/home/coderbot/.steam/steam/steamapps/common/Team Fortress 2/tf/demos/2017-12-23_16-43-13.dem";
//const PATH: &str = "/home/coderbot/.steam/steam/steamapps/common/Team Fortress 2/tf/demos/2018-07-28_22-43-39.dem";
//const PATH: &str = "/home/coderbot/.steam/steam/steamapps/common/Team Fortress 2/tf/demos/2016-12-07_18-25-34.dem";
//const PATH: &str = "/home/coderbot/Programming/Rust IntelliJ/demoman/test_data/2013-04-10-Granary.dem";
//const PATH: &str = "/home/coderbot/Programming/Rust IntelliJ/demoman/test_data/2013-02-19-ctf_haunt_b2.dem";
const PATH: &str = "/home/coderbot/Programming/Rust IntelliJ/demoman/test_data/2012-06-29-Dustbowl.dem";
const USE_OLD_VOICEINIT: bool = true;

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

fn parse_update(data: Vec<u8>, demo: &DemoHeader) {
	let data = Bits::from_bytes(data);
	let mut bits = data.reader();

	assert!(demo.network_protocol > 10, "Network protocols less than 10 do not have fixed_time and fixed_time_stdev in Tick, this is not handled yet!");

	while bits.remaining_bits() >= 6 {
		let id = bits.read_bits(6);

		let kind = PacketKind::from_id(id as u8).expect("Packet ID cannot be greater than 31");

		print!("  {:>17} | ", format!("{:?}", kind));

		match kind {
			PacketKind::Nop               => {
				if bits.remaining_bits() >= 6 {
					println!("[Warning: Nop packet found in the middle of an update, this usually means that a packet was improperly parsed]");
				} else {
					println!("Nop");
				}
			},
			PacketKind::Disconnect        => unimplemented!(),
			PacketKind::TransferFile      => println!("{:?}", TransferFile::parse(&mut bits)),
			PacketKind::Tick              => println!("{:?}", Tick::parse(&mut bits)),
			PacketKind::StringCommand     => println!("{:?}", bits.read_string()),
			PacketKind::SetCvars          => {
				let SetCvars(cvars) = SetCvars::parse(&mut bits);

				println!("{} cvars", cvars.len());

				for &(ref cvar, ref value) in &cvars {
					println!("  {:>17} : {:?} = {:?}", "", cvar, value);
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
				let update = UpdateStringTable::parse(&mut bits);

				println!("Table: {}, Entries: {}, Bits: {}", update.table_id, update.entries, update.data.bits_len());
			},
			PacketKind::VoiceInit         => {
				if USE_OLD_VOICEINIT {
					/// VoiceInit message that is missing an extra 16-bit field.
					#[derive(Debug, Clone)]
					pub struct VoiceInitOld {
						pub codec: String,
						pub quality: u8
					}

					impl VoiceInitOld {
						pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
							VoiceInitOld {
								codec:   bits.read_string().unwrap(),
								quality: bits.read_u8()
							}
						}
					}

					println!("{:?}", VoiceInitOld::parse(&mut bits));
				} else {
					println!("{:?}", VoiceInit::parse(&mut bits));
				}

				// TODO: Changed recently: Medics demo has different format...
			},
			PacketKind::VoiceData         => {
				let voice_data = VoiceData::parse(&mut bits);

				println!("Sender: {}, Proximity: {}, Bits: {}", voice_data.sender, voice_data.proximity, voice_data.data.bits_len());
			},
			PacketKind::HltvControl      => unimplemented!(),
			PacketKind::PlaySound        => match PlaySound::parse(&mut bits) {
				PlaySound::Reliable(data)             => println!("Reliable: {} bits", data.bits_len()),
				PlaySound::Unreliable { sounds, all } => println!("Unreliable: {} sounds, {} bits", sounds, all.bits_len())
			},
			PacketKind::SetEntityView    => println!("Entity: {}", bits.read_bits(11)),
			PacketKind::FixAngle         => println!("{:?}", FixAngle::parse(&mut bits)),
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
				let user_message = UserMessage::parse(&mut bits);

				println!("Channel: {}, Bits: {}", user_message.channel, user_message.data.bits_len());
			},
			PacketKind::EntityMessage    => {
				let entity_message = EntityMessage::parse(&mut bits);

				println!("Entity: {}, Class: {}, Bits: {}", entity_message.entity, entity_message.class, entity_message.data.bits_len());
			},
			PacketKind::GameEvent        => {
				let GameEvent(payload) = GameEvent::parse(&mut bits);

				if payload.bits_len() < 9 {
					println!("Error: Too small! Bits: {}", payload.bits_len());
					break;
				}

				let id = payload.reader().read_bits(9);

				println!("Event ID: {}, Bits: {}", id, payload.bits_len());
			},
			PacketKind::Entities         => {
				let entities = Entities::parse(&mut bits);

				println!("Entries: {} updated / {} max, Baseline: {} Update Baseline: {}, Delta From Tick: {:?}, Bits: {}", entities.updated, entities.max_entries, entities.baseline, entities.update_baseline, entities.delta_from_tick, entities.data.bits_len());

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
			PacketKind::Prefetch         => println!("{:?}", Prefetch::parse(&mut bits)),
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

	if bits.remaining_bits() >= 6 {
		println!(" === SOME PACKETS NOT PARSED ==");
		::std::process::exit(0);
	}
}