pub mod game_events;
pub mod string_table;

use demo::bits::{BitReader, Bits};
use std::io::Read;

/// Only needed on old demos, appears to be v22 and below. V23 not checked though.
pub const COMPATIBILITY_USE_FIXED_CREATESTRINGTABLE_LENGTH_FIELD: bool = true;
/// True in modern demos. v22 and below (v23?) don't have a type identifier, however.
pub const PREFETCH_HAS_TYPE_IDENTIFIER: bool = false;

type EntityId = u16;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PacketKind {
	Nop,
	Disconnect,
	TransferFile,
	Tick,
	StringCommand,
	SetCvars,
	SignonState,
	Print,
	ServerInfo,
	DataTable,
	ClassInfo,
	Pause,
	CreateStringTable,
	UpdateStringTable,
	VoiceInit,
	VoiceData,
	HltvControl,
	PlaySound,
	SetEntityView,
	FixAngle,
	CrosshairAngle,
	Decal,
	TerrainMod,
	UserMessage,
	EntityMessage,
	GameEvent,
	Entities,
	TempEntities,
	Prefetch,
	PluginMenu,
	GameEventList,
	GetCvar
}

impl PacketKind {
	pub fn from_id(id: u8) -> Option<Self> {
		Some(match id {
			0  => PacketKind::Nop,
			1  => PacketKind::Disconnect,
			2  => PacketKind::TransferFile,
			3  => PacketKind::Tick,
			4  => PacketKind::StringCommand,
			5  => PacketKind::SetCvars,
			6  => PacketKind::SignonState,
			7  => PacketKind::Print,
			8  => PacketKind::ServerInfo,
			9  => PacketKind::DataTable,
			10 => PacketKind::ClassInfo,
			11 => PacketKind::Pause,
			12 => PacketKind::CreateStringTable,
			13 => PacketKind::UpdateStringTable,
			14 => PacketKind::VoiceInit,
			15 => PacketKind::VoiceData,
			16 => PacketKind::HltvControl,
			17 => PacketKind::PlaySound,
			18 => PacketKind::SetEntityView,
			19 => PacketKind::FixAngle,
			20 => PacketKind::CrosshairAngle,
			21 => PacketKind::Decal,
			22 => PacketKind::TerrainMod,
			23 => PacketKind::UserMessage,
			24 => PacketKind::EntityMessage,
			25 => PacketKind::GameEvent,
			26 => PacketKind::Entities,
			27 => PacketKind::TempEntities,
			28 => PacketKind::Prefetch,
			29 => PacketKind::PluginMenu,
			30 => PacketKind::GameEventList,
			31 => PacketKind::GetCvar,
			_ => return None
		})
	}
}

pub enum Packet {
	Nop,
	Disconnect,          // TODO
	TransferFile         (TransferFile),
	Tick                 (Tick),
	StringCommand        (String),
	SetCvars             (SetCvars),
	SignonState          (SignonState),
	Print                (String),
	ServerInfo           (ServerInfo),
	DataTable,           // TODO
	ClassInfo            (ClassInfo),
	Pause,               // TODO
	CreateStringTable    (CreateStringTable),
	UpdateStringTable    (UpdateStringTable),
	VoiceInit            (VoiceInit),
	VoiceData            (VoiceData),
	HltvControl,         // UNUSED
	PlaySound            (PlaySound),
	SetEntityView        (EntityId),
	FixAngle             (FixAngle),
	CrosshairAngle,      // TODO
	Decal                (Decal),
	TerrainModification, // UNUSED
	UserMessage          (UserMessage),
	EntityMessage        (EntityMessage),
	GameEvent            (GameEvent),
	Entities,            // TODO
	TempEntities,        // TODO
	Prefetch             (Prefetch),
	PluginMenu,          // TODO
	GameEventList        (game_events::GameEventList),
	GetCvar              // TODO
}

impl Packet {
	pub fn parse_with_kind<R>(bits: &mut BitReader<R>, kind: PacketKind) -> Self where R: Read {
		match kind {
			PacketKind::Nop               => Packet::Nop,
			PacketKind::Disconnect        => unimplemented!(),
			PacketKind::TransferFile      => Packet::TransferFile     (TransferFile::parse(bits)),
			PacketKind::Tick              => Packet::Tick             (Tick::parse(bits)),
			PacketKind::StringCommand     => Packet::StringCommand    (bits.read_string().unwrap()),
			PacketKind::SetCvars          => Packet::SetCvars         (SetCvars::parse(bits)),
			PacketKind::SignonState       => Packet::SignonState      (SignonState::parse(bits)),
			PacketKind::Print             => Packet::Print            (bits.read_string().unwrap()),
			PacketKind::ServerInfo        => Packet::ServerInfo       (ServerInfo::parse(bits)),
			PacketKind::DataTable         => unimplemented!(),
			PacketKind::ClassInfo         => Packet::ClassInfo        (ClassInfo::parse(bits)),
			PacketKind::Pause             => unimplemented!(),
			PacketKind::CreateStringTable => Packet::CreateStringTable(CreateStringTable::parse(bits)),
			PacketKind::UpdateStringTable => Packet::UpdateStringTable(UpdateStringTable::parse(bits)),
			PacketKind::VoiceInit         => Packet::VoiceInit        (VoiceInit::parse(bits)),
			PacketKind::VoiceData         => Packet::VoiceData        (VoiceData::parse(bits)),
			PacketKind::HltvControl       => unimplemented!(),
			PacketKind::PlaySound         => Packet::PlaySound        (PlaySound::parse(bits)),
			PacketKind::SetEntityView     => Packet::SetEntityView    (bits.read_bits(11) as u16),
			PacketKind::FixAngle          => Packet::FixAngle         (FixAngle::parse(bits)),
			PacketKind::CrosshairAngle    => unimplemented!(),
			PacketKind::Decal             => Packet::Decal            (Decal::parse(bits)),
			PacketKind::TerrainMod        => unimplemented!(),
			PacketKind::UserMessage       => Packet::UserMessage      (UserMessage::parse(bits)),
			PacketKind::EntityMessage     => Packet::EntityMessage    (EntityMessage::parse(bits)),
			PacketKind::GameEvent         => Packet::GameEvent        (GameEvent::parse(bits)),
			PacketKind::Entities          => unimplemented!(),
			PacketKind::TempEntities      => unimplemented!(),
			PacketKind::Prefetch          => Packet::Prefetch         (Prefetch::parse(bits)),
			PacketKind::PluginMenu        => unimplemented!(),
			PacketKind::GameEventList     => Packet::GameEventList    (game_events::GameEventList::parse(bits)),
			PacketKind::GetCvar           => unimplemented!()
		}
	}
}

#[derive(Debug, Clone)]
pub struct TransferFile {
	pub transfer_id: u32,
	pub name: String,
	/// If this is false, then the file is denied. Otherwise, it is requested.
	pub request_or_deny: bool
}

impl TransferFile {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		TransferFile {
			transfer_id: bits.read_u32(),
			name: bits.read_string().unwrap(),
			request_or_deny: bits.read_bit()
		}
	}
}

#[derive(Debug, Clone)]
pub struct Tick {
	/// Server-side tick number.
	pub number: u32,
	/// Tick time in seconds, times 100000
	pub fixed_time: u16,
	/// Standard deviation of the tick time in seconds, times 100000
	pub fixed_time_stdev: u16
}

impl Tick {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		Tick {
			number: bits.read_u32(),
			fixed_time: bits.read_u16(),
			fixed_time_stdev: bits.read_u16()
		}
	}
}

#[derive(Debug, Clone)]
pub struct SetCvars(pub Vec<(String, String)>);

impl SetCvars {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		let count = bits.read_u8();
		let mut cvars = Vec::new();

		for _ in 0..count {
			cvars.push((bits.read_string().unwrap(), bits.read_string().unwrap()));
		}

		SetCvars(cvars)
	}
}

#[derive(Debug, Clone)]
pub enum SignonStateKind {
	None,
	Challenge,
	Connected,
	New,
	PreSpawn,
	Spawn,
	Full,
	ChangeLevel
}

impl SignonStateKind {
	fn from_id(id: u8) -> Result<Self, u8> {
		Ok(match id {
			0 => SignonStateKind::None,
			1 => SignonStateKind::Challenge,
			2 => SignonStateKind::Connected,
			3 => SignonStateKind::New,
			4 => SignonStateKind::PreSpawn,
			5 => SignonStateKind::Spawn,
			6 => SignonStateKind::Full,
			7 => SignonStateKind::ChangeLevel,
			_ => return Err(id)
		})
	}
}

#[derive(Debug, Clone)]
pub struct SignonState {
	pub state: Result<SignonStateKind, u8>,
	pub server_count: u32
}

impl SignonState {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		SignonState {
			state: SignonStateKind::from_id(bits.read_u8()),
			server_count: bits.read_u32()
		}
	}
}

#[derive(Debug, Clone)]
pub struct ServerInfo {
	/// Network Protocol ID. Same as the network_protocol value in the DemoHeader.
	pub network_protocol: u16,
	/// Unknown meaning.
	pub server_count: u32,
	/// Unknown: Does this indicate the presence of HLTV, or does this indicate that the connection is through HLTV
	pub hltv: bool,
	/// Whether the server is a dedicated or listen server.
	pub dedicated: bool,
	/// CRC of the client DLL file. -1 seems to indicate that there is no CRC.
	pub client_dll_crc: u32,
	/// The maximum amount of "classes". This amount matches the count of the class mappings found in the DataTables.
	pub max_classes: u16,
	pub _unknown0: [u8; 16],
	/// Player slot that the client now occupies.
	pub slot: u8,
	/// Maximum amount of clients that the server can handle.
	pub max_clients: u8,
	/// How many seconds a single tick takes. The server's target TPS is `1 / tick_interval`.
	pub tick_interval: f32,
	/// Identifier of the OS that this server is running on.
	pub os: u8,
	/// Game directory. TF2's directory is "tf".
	pub game_directory: String,
	/// Map name. Example: `ctf_2fort`
	pub map: String,
	/// Sky name. Example; `sky_tf2_04`
	pub sky: String,
	/// Host name. Not an address. Instead, this is the human readable name the server prefers to go by.
	pub hostname: String,
	/// Unknown value. Supposedly not present before network_protocol 16.
	pub _unknown1: bool
}

impl ServerInfo {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		ServerInfo {
			network_protocol: bits.read_u16(),
			server_count: bits.read_u32(),
			hltv: bits.read_bit(),
			dedicated: bits.read_bit(),
			client_dll_crc: bits.read_u32(),
			max_classes: bits.read_u16(),
			_unknown0: [
				bits.read_u8(), bits.read_u8(), bits.read_u8(), bits.read_u8(),
				bits.read_u8(), bits.read_u8(), bits.read_u8(), bits.read_u8(),
				bits.read_u8(), bits.read_u8(), bits.read_u8(), bits.read_u8(),
				bits.read_u8(), bits.read_u8(), bits.read_u8(), bits.read_u8()
			],
			slot: bits.read_u8(),
			max_clients: bits.read_u8(),
			tick_interval: bits.read_f32(),
			os: bits.read_u8(),
			game_directory: bits.read_string().unwrap(),
			map: bits.read_string().unwrap(),
			sky: bits.read_string().unwrap(),
			hostname: bits.read_string().unwrap(),
			_unknown1: bits.read_bit()
		}
	}
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
	pub classes: u16,
	pub info: Option<()>
}

impl ClassInfo {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		let classes = bits.read_u16();
		let no_parse = bits.read_bit();

		assert!(no_parse, "Don't know how to parse the body of ClassInfo!");

		ClassInfo {
			classes,
			info: None
		}
	}
}

#[derive(Debug, Clone)]
pub struct CreateStringTable {
	pub name: String,
	pub max_entries: u16,
	pub entries: u16,
	pub fixed_userdata_size: Option<(u16, u8)>,
	pub compressed: bool,
	pub data: Bits
}

impl CreateStringTable {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		let name = bits.read_string().unwrap();
		let max_entries = bits.read_u16();

		assert_ne!(max_entries, 0);

		let index_bits = (16 - max_entries.leading_zeros()) as u8 - 1;
		let entries = bits.read_bits(index_bits + 1) as u16;
		let bits_len = if !COMPATIBILITY_USE_FIXED_CREATESTRINGTABLE_LENGTH_FIELD { bits.read_var_u32() } else { bits.read_bits(20) };

		// Size and Bits Size
		let fixed_userdata_size = if bits.read_bit()  {
			Some((bits.read_bits(12) as u16, bits.read_bits(4) as u8))
		} else {
			None
		};

		let compressed = bits.read_bit();

		let data = Bits::copy_into(bits, bits_len as usize);

		CreateStringTable { name, max_entries, entries, fixed_userdata_size, compressed, data }
	}
}

#[derive(Debug, Clone)]
pub struct UpdateStringTable {
	pub table_id: u8,
	pub entries:  u16,
	pub data:     Bits
}

impl UpdateStringTable {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		UpdateStringTable {
			table_id: bits.read_bits(5) as u8,
			entries: if bits.read_bit() { bits.read_u16() } else { 1 },
			data: {
				let bits_len = bits.read_bits(20) as usize;
				Bits::copy_into(bits, bits_len)
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct VoiceInit {
	pub codec: String,
	pub quality: u8,
	pub unknown: u16
}

impl VoiceInit {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		VoiceInit {
			codec:   bits.read_string().unwrap(),
			quality: bits.read_u8(),
			unknown: bits.read_u16()
		}
	}
}

#[derive(Debug, Clone)]
pub struct VoiceData {
	pub sender:    u8,
	pub proximity: u8,
	pub data:      Bits
}

impl VoiceData {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		VoiceData {
			sender: bits.read_u8(),
			proximity: bits.read_u8(),
			data: {
				let len = bits.read_u16();

				Bits::copy_into(bits, len as usize)
			}
		}
	}
}

#[derive(Debug, Clone)]
pub enum PlaySound {
	Reliable   (Bits),
	Unreliable { sounds: u8, all: Bits }
}

impl PlaySound {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		let reliable = bits.read_bit();

		if reliable {
			let bit_len = bits.read_u8();

			PlaySound::Reliable(Bits::copy_into(bits, bit_len as usize))
		} else {
			let sounds = bits.read_u8();
			let bit_len = bits.read_u16();

			PlaySound::Unreliable { sounds, all: Bits::copy_into(bits, bit_len as usize) }
		}
	}
}

#[derive(Debug, Clone)]
pub struct FixAngle {
	relative: bool,
	angles: (u16, u16, u16)
}

impl FixAngle {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		FixAngle {
			relative: bits.read_bit(),
			angles: (
				bits.read_u16(),
				bits.read_u16(),
				bits.read_u16()
			)
		}
	}
}

#[derive(Debug, Clone)]
pub struct Decal {
	pub position: (f32, f32, f32),
	pub decal_index: u16,
	pub entity_index: EntityId,
	pub model_index: u16,
	pub low_priority: bool
}

impl Decal {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		let position = bits.read_vec3();
		let decal_index = bits.read_bits(9) as u16;

		let (entity_index, model_index) = if bits.read_bit() {
			(bits.read_bits(11) as u16, bits.read_bits(11) as u16)
		} else {
			(0, 0)
		};

		let low_priority = bits.read_bit();

		Decal { position, decal_index, entity_index, model_index, low_priority }
	}
}

#[derive(Debug, Clone)]
pub struct UserMessage {
	pub channel: u8,
	pub data:    Bits
}

impl UserMessage {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		UserMessage {
			channel: bits.read_u8(),
			data: {
				let bits_len = bits.read_bits(11) as usize;
				Bits::copy_into(bits, bits_len)
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct EntityMessage {
	pub entity: EntityId,
	pub class:  u16,
	pub data:   Bits
}

impl EntityMessage {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		EntityMessage {
			entity: bits.read_bits(11) as u16,
			class:  bits.read_bits(9) as u16,
			data: {
				let bits_len = bits.read_bits(11) as usize;
				Bits::copy_into(bits, bits_len)
			}
		}
	}
}

// First 9 bits are the event ID
#[derive(Debug, Clone)]
pub struct GameEvent(pub Bits);

impl GameEvent {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		let bits_len = bits.read_bits(11) as usize;

		GameEvent(Bits::copy_into(bits, bits_len))
	}
}

#[derive(Debug, Clone)]
pub struct Entities {
	pub max_entries: u16,
	pub delta_from_tick: Option<u32>,
	pub baseline: bool,
	pub updated: u16,
	pub update_baseline: bool,
	pub data: Bits
}

impl Entities {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		let max_entries = bits.read_bits(11) as u16;

		let delta_from_tick = if bits.read_bit() {
			Some(bits.read_u32())
		} else {
			None
		};

		let baseline = bits.read_bit();
		let updated = bits.read_bits(11) as u16;
		let bits_len = bits.read_bits(20) as usize;
		let update_baseline = bits.read_bit();

		Entities {
			max_entries,
			delta_from_tick,
			baseline,
			updated,
			update_baseline,
			data: Bits::copy_into(bits, bits_len)
		}
	}
}

pub struct TempEntities;

impl TempEntities {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		TempEntities
	}
}

#[derive(Debug, Clone)]
pub struct Prefetch {
	// TODO: Appears to be a type identifier.
	pub unknown: bool,
	pub id: u16
}

impl Prefetch {
	pub fn parse<R>(bits: &mut BitReader<R>) -> Self where R: Read {
		Prefetch {
			unknown: if PREFETCH_HAS_TYPE_IDENTIFIER { bits.read_bit() } else { false },
			id:      bits.read_bits(13) as u16
		}
	}
}
