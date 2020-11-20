mod data;

use super::Handler;
use demo::packets::{Packet, GameEvent};
use demo::packets::game_events::{GameEventList, GameEventInfo, Kind};
use data::Value;

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
