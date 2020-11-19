extern crate dem;
extern crate byteorder;
extern crate snap;

use dem::demo::header::{self, DemoHeader};
use dem::demo::bits::Bits;
use dem::packets::{ProtocolVersion, PacketKind, Packet, PlaySound, SetCvars, GameEvent};
use dem::packets::game_events::{GameEventList, GameEventInfo, Kind};
use dem::packets::string_table::Extra;
use dem::demo::frame::{Frame, FramePayload};

use std::io::{BufReader, Read, Seek, SeekFrom};
use std::fs::File;

const MAX_PARSED_PACKETS: usize = 4096;
//const MAX_PARSED_PACKETS: usize = 4_000_000_000;
const SHOW_STRING_TABLES: bool = false;
const SHOW_DATA_TABLES: bool = false;
const SHOW_STRING_TABLE_CONTENTS: bool = false;
const SHOW_FRAME_HEADER_SPAM: bool = false;
const SHOW_COMMANDS: bool = false;

trait Handler {
	fn packet(&mut self, packet: Packet);
}

fn main() {
	let path = match std::env::args().skip(1).next() {
		Some(path) => path,
		None => {
			eprintln!("Usage: demoman <file>");
			return;
		}
	};

	let file = match File::open(path) {
		Ok(file) => file,
		Err(err) => {
			eprintln!("couldn't open demo file for reading: {}", err);
			eprintln!("note: Make sure you typed the path correctly and have the right permissions");
			return;
		}
	};

	let mut file = BufReader::new(file);

	let mut buf = [0; header::HEADER_LENGTH];
	file.read(&mut buf[0..]).unwrap();

	let demo = match DemoHeader::parse(&buf) {
		Ok(header) => header,
		Err(err) => {
			eprintln!("error while reading demo file header: {:?}", err);
			eprintln!("note: Demo file had incorrect magic value, expected HL2DEMO\\0 at start of file");
			eprintln!("note: This doesn't appear to be a valid demo file");

			return
		}
	};

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

	//let mut handler = ShowGameEvents { list: None };
	//let mut handler = PrintAll;
	let mut handler = DumpVoiceData;

	// Iterate over a limited amount of packets
	for _ in 0..MAX_PARSED_PACKETS {
		let offset = file.seek(SeekFrom::Current(0)).unwrap();

		if offset == signon_end {
			println!();
			println!("-- END OF SIGNON DATA --");
			println!();
		}

		let frame = Frame::parse(&mut file).unwrap();

		if SHOW_FRAME_HEADER_SPAM {
			print!("T: {} ", frame.tick);
		}

		match frame.payload {
			FramePayload::SignonUpdate(update) | FramePayload::Update(update) => {
				if SHOW_FRAME_HEADER_SPAM {
					println!("| Update ({} packet bytes) [OFFS:{}]", update.packets.len(), file.seek(SeekFrom::Current(0)).unwrap());
				}

				parse_update(update.packets, &demo, &mut handler);
			},
			FramePayload::TickSync => println!("| Tick Sync"),
			FramePayload::ConsoleCommand(command) => if SHOW_COMMANDS { println!("> {}", command) },
			FramePayload::UserCmdDelta { sequence: _, frame } => {
				let _frame = frame.parse().unwrap();
				/*println!("| UserCmdDelta (hidden)")*/()
			},
			FramePayload::DataTables(tables) => {
				let tables = tables.parse().unwrap();

				if SHOW_DATA_TABLES { println!("| Data Tables - {} tables, {} class links", tables.tables.len(), tables.links.len()) }
			},
			FramePayload::Stop => {
				println!("| Stop");
				break;
			},
			FramePayload::StringTables(tables) => {
				let tables = tables.parse().unwrap();

				if !SHOW_STRING_TABLES {
					continue;
				}

				println!("| String Tables - {} tables", tables.0.len());

				for (index, &(ref name, ref pair)) in tables.0.iter().enumerate() {
					print!("  #{} | {}: {} primary strings, ", index, name, pair.primary.strings.len());
					match &pair.client {
						&Some(ref table) => println!("{} client strings", table.strings.len()),
						&None => println!("no client strings")
					}

					if !SHOW_STRING_TABLE_CONTENTS {
						continue;
					}

					for (index, &(ref string, ref extra)) in pair.primary.strings.iter().enumerate() {
						print!("    #{}: {} ", index, string);
						match extra {
							&Extra::Bits { count, data } => println!("= (bit count: {}, bit data: {})", count, data),
							&Extra::Bytes(ref bytes) => {
								print!("= ");

								for &byte in bytes {
									print!("{:02X} ", byte);
								}

								println!()
							},
							&Extra::None => println!()
						}
					}
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


struct DumpVoiceData;

impl Handler for DumpVoiceData {
	fn packet(&mut self, packet: Packet) {
		match packet {
			Packet::VoiceInit(packet) => {
				println!("{:?}", packet)
			},
			Packet::VoiceData(packet) => {
				//println!("{}", packet.data.raw_bytes().len());
				print!("[Sender: {}, Proximity: {}, Bytes: {}] ", packet.sender, packet.proximity, packet.data.raw_bytes().len());

				if packet.data.bits_len() == 0 {
					println!("[Voice Data Ack]");
					return;
				}

				for &byte in packet.data.raw_bytes().iter()/*.take(20)*/ {
					print!("{:02X} ", byte);
				}

				println!();
				//println!();
			},
			_ => ()
		}
	}
}

struct ShowGameEvents {
	list: Option<Vec<GameEventInfo>>
}

impl Handler for ShowGameEvents {
	fn packet(&mut self, packet: Packet) {
		match packet {
			Packet::GameEventList(GameEventList(event_list)) => {
				println!("List of game events:");

				for &GameEventInfo { index, ref name, properties: _ } in &event_list {
					println!("Index: {}, Name: {}", index, name);
				}

				self.list = Some(event_list);
			},
			Packet::GameEvent(GameEvent(payload)) => {
				if payload.bits_len() < 9 {
					println!("Error: Too small! Bits: {}", payload.bits_len());
					return;
				}

				//println!("GameEvent | Bits: {}", payload.bits_len() - 9);

				let mut reader = payload.reader();
				let id = reader.read_bits(9).unwrap() as u16;
				let mut game_event = None;

				if self.list.is_none() {
					println!("Error: Recieved GameEvent ID {} before recieving the GameEvent list, cannot decode!", id);
					return;
				}

				for event in self.list.as_ref().unwrap().iter() {
					if event.index == id {
						game_event = Some(event);
						break;
					}
				}

				let game_event = game_event.expect("Bad event index");

				//println!("  Index: {}", id, game_event.name);

				use dem::payload::game_events::Value;

				let mut values = ::std::collections::HashMap::new();

				for &(kind, ref name) in &game_event.properties {
					let value = match kind {
						Kind::End    => unreachable!(),
						Kind::Str    => Value::Str (reader.read_string().unwrap()),
						Kind::F32    => Value::F32 (reader.read_f32().unwrap()),
						Kind::I32    => Value::I32 (reader.read_i32().unwrap()),
						Kind::I16    => Value::I16 (reader.read_i16().unwrap()),
						Kind::U8     => Value::U8  (reader.read_u8().unwrap()),
						Kind::Bool   => Value::Bool(reader.read_bit().unwrap()),
						Kind::Unused => unimplemented!()
					};

					values.insert(name.to_owned(), value);



					/*match kind {
						Kind::End    => unreachable!(),
						Kind::Str    => println!("{:?}", reader.read_string().unwrap()),
						Kind::F32    => println!("{}", reader.read_f32()),
						Kind::I32    => println!("{}", reader.read_i32()),
						Kind::I16    => println!("{}", reader.read_i16()),
						Kind::U8     => println!("{}", reader.read_u8()),
						Kind::Bool   => println!("{}", reader.read_bit()),
						Kind::Unused => unimplemented!()
					}*/
				}

				println!("{{");
				println!("  \"event\": {:?},", game_event.name);
				println!("  \"values\": {{");

				let mut before = false;

				for (key, value) in &values {
					if before {
						println!(",");
					}

					print!("    \"{}\": ", key);
					match *value {
						Value::Str(ref name) => print!("\"{}\"", name.replace('\"', "\\\"")),
						Value::F32(value) => print!("{}", value),
						Value::I32(value) => print!("{}", value),
						Value::I16(value) => print!("{}", value),
						Value::U8(value) => print!("{}", value),
						Value::Bool(value) => print!("{}", value)
					}

					before = true;
				}
				println!();

				println!("  }}");
				println!("}},");

				/*let values = ::dem::payload::game_events::GameEventData(values);

				match game_event.name.as_ref() {
					"player_hurt" => (),
					"player_healed" => (),
					"post_inventory_application" => (),
					"player_healonhit" => (),
					"player_death" => {
						let attacker = values.get_i16("attacker").unwrap();
						let assister = values.get_i16("assister").unwrap();
						let victim = values.get_i16("userid").unwrap();
						let weapon = values.get_str("weapon").unwrap();

						// not exhaustive

						if assister != -1 {
							println!("{} + {} killed {} using {} (assist)", attacker, assister, victim, weapon);
						} else {
							println!("{} killed {} using {}", attacker, victim, weapon);
						}
					},
					"player_disconnect" => {
						let userid = values.get_i16("userid").unwrap();
						let name = values.get_str("name").unwrap();
						let networkid = values.get_str("networkid").unwrap();
						let bot = values.get_i16("bot").unwrap();
						let reason = values.get_str("reason").unwrap();

						println!("{} left the game ({}) [userid {}, networkid {}]", name, reason, userid, networkid)
					},
					other => println!("{}", other)
				}*/

				//println!("{}", game_event.name);
			},
			_ => ()
		}
	}
}

struct PrintAll;
impl Handler for PrintAll {
	fn packet(&mut self, packet: Packet) {
		print!("  {:>17} | ", format!("{:?}", packet.kind()));

		match packet {
			Packet::Nop                       => println!(),
			Packet::Disconnect                => unimplemented!(),
			Packet::TransferFile(packet)      => println!("{:?}", packet),
			Packet::Tick(packet)              => println!("{:?}", packet),
			Packet::StringCommand(packet)     => println!("{:?}", packet),
			Packet::SetCvars(packet)          => {
				let SetCvars(cvars) = packet;

				println!("{} cvars", cvars.len());

				for &(ref cvar, ref value) in &cvars {
					println!("  {:>17} : {:?} = {:?}", "", cvar, value);
				}
			},
			Packet::SignonState(packet)       => println!("{:?}", packet),
			Packet::Print(packet)             => println!("{:?}", packet),
			Packet::ServerInfo(packet)        => println!("{:?}", packet),
			Packet::DataTable                 => unimplemented!(),
			Packet::ClassInfo(packet)         => println!("{:?}", packet),
			Packet::Pause(paused)             => println!("Is Paused: {}", paused),
			Packet::CreateStringTable(packet) => println!("Table: {}, Entries: {} / {:?}, Fixed Userdata Size: {:?}, Bits: {}", packet.name, packet.entries, packet.max_entries, packet.fixed_userdata_size, packet.data.bits_len()),
			Packet::UpdateStringTable(packet) => println!("Table: {}, Entries: {}, Bits: {}", packet.table_id, packet.entries, packet.data.bits_len()),
			Packet::VoiceInit(packet)         => println!("{:?}", packet),
			Packet::VoiceData(packet)         => println!("Sender: {}, Proximity: {}, Bits: {}", packet.sender, packet.proximity, packet.data.bits_len()),
			Packet::HltvControl               => unimplemented!(),
			Packet::PlaySound(packet)         => match packet {
				PlaySound::Reliable(data)             => println!("Reliable: {} bits", data.bits_len()),
				PlaySound::Unreliable { sounds, all } => println!("Unreliable: {} sounds, {} bits", sounds, all.bits_len())
			},
			Packet::SetEntityView(packet)    => println!("{}", packet),
			Packet::FixAngle(packet)         => println!("{:?}", packet),
			Packet::CrosshairAngle(packet)   => println!("{:?}", packet),
			Packet::Decal(packet)            => println!("{:?}", packet),
			Packet::TerrainMod               => unimplemented!(),
			Packet::UserMessage(packet)      => println!("Channel: {}, Bits: {}", packet.channel, packet.data.bits_len()),
			Packet::EntityMessage(packet)    => println!("Entity: {}, Class: {}, Bits: {}", packet.entity, packet.class, packet.data.bits_len()),
			Packet::GameEvent(packet)        => {
				let GameEvent(payload) = packet;

				if payload.bits_len() < 9 {
					println!("Error: Too small! Bits: {}", payload.bits_len());
					return;
				}

				let id = payload.reader().read_bits(9).unwrap();

				println!("Event ID: {}, Bits: {}", id, payload.bits_len() - 9);
			},
			Packet::Entities(packet)         => {
				println!("Entries: {} updated / {} max, Baseline: {} Update Baseline: {}, Delta From Tick: {:?}, Bits: {}", packet.updated, packet.max_entries, packet.baseline, packet.update_baseline, packet.delta_from_tick, packet.data.bits_len());

				// TODO: Parse entities properly

				/*let mut bits = packet.data.reader();

				#[derive(Debug, Eq, PartialEq, Copy, Clone)]
				enum UpdateType {
					EnterPvs,
					LeavePvs,
					Delete,
					Delta,
					// TODO Finished,
					Preserve
				}

				let mut remaining_headers = packet.updated;

				'outer:
					loop {
					remaining_headers -= 1;
					let is_entity = remaining_headers >= 0;

					let base_update_type = if is_entity {
						Some(match (bits.read_bit().unwrap(), bits.read_bit().unwrap()) {
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
							// TODO: UpdateType::Finished => unreachable!()
						}

						println!("  {:>17} : Update Type: {:?}", "", update_type);

						if update_type != UpdateType::Preserve {
							break 'outer;
						}
					}
				}*/
			},
			Packet::TempEntities(packet)     => println!("Count: {}, Bits: {}", packet.count, packet.data.bits_len()),
			Packet::Prefetch(packet)         => println!("{:?}", packet),
			Packet::PluginMenu(packet)       => println!("Kind: {}, Bytes: {}", packet.kind, packet.data.len()),
			Packet::GameEventList(packet)    => println!("{} events not shown", packet.0.len()),
			Packet::GetCvar                  => {
				// TODO: BROKEN?

				//println!("Cookie: {}, CVar: {:?}", bits.read_u32(), bits.read_string());

				println!("  Don't know how to handle a GetCvarValue!");
			}
		}
	}
}

fn parse_update<H>(data: Vec<u8>, demo: &DemoHeader, handler: &mut H) where H: Handler {
	let data = Bits::from_bytes(data);
	let mut bits = data.reader();

	if demo.network_protocol < 10 {
		unimplemented!("Network protocols less than 10 do not have fixed_time and fixed_time_stdev in Tick, this is not handled yet!");
	}

	let version = ProtocolVersion(demo.network_protocol as u32);

	while bits.has_remaining(version.packet_kind_bits() as usize) {
		let id = bits.read_bits(version.packet_kind_bits()).unwrap();

		let kind = PacketKind::from_id(id as u8).expect("Packet ID cannot be greater than 31");

		handler.packet(Packet::parse_with_kind(&mut bits, kind, version).unwrap());
	}
}