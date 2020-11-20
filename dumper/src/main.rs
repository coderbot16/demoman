pub mod game_events;
pub mod voice;
pub mod print_all;

use demo::header::{self, DemoHeader};
use bitstream::Bits;
use demo::packets::{ProtocolVersion, PacketKind, Packet};
use demo::string_table::{StringTables, Extra};
use demo::frame::{Frame, FramePayload};

use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::fs::File;

const MAX_PARSED_PACKETS: usize = 4096;
//const MAX_PARSED_PACKETS: usize = 4_000_000_000;
const SHOW_STRING_TABLES: bool = false;
const SHOW_DATA_TABLES: bool = false;
const SHOW_STRING_TABLE_CONTENTS: bool = false;
const SHOW_FRAME_HEADER_SPAM: bool = false;
const SHOW_COMMANDS: bool = false;

pub trait Handler {
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

	if let Err(err) = file.read_exact(&mut buf[0..]) {
		eprintln!("error while reading demo file header: {:?}", err);

		if err.kind() == io::ErrorKind::UnexpectedEof {
			eprintln!("note: Demo file is too short to possibly be a valid demo file")
		}

		return
	}

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
	let mut handler = voice::DumpVoiceData;

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
				handle_string_table(tables.parse().unwrap())
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

fn handle_string_table(tables: StringTables) {
	if !SHOW_STRING_TABLES {
		return;
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
