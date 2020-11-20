use demo::packets::Packet;
use super::Handler;

pub struct DumpVoiceData;

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
