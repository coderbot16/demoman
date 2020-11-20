use super::Handler;
use demo::packets::{Packet, SetCvars, GameEvent, PlaySound};

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
