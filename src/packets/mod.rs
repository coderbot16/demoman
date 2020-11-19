pub mod game_events;
pub mod string_table;

use crate::demo::bits::{BitReader, Bits};
use crate::demo::parse::ParseError;

type EntityId = u16;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ProtocolVersion(pub u32);

impl ProtocolVersion {
	pub fn packet_kind_bits(self) -> u8 {
		// TODO: 16 is just a guess, I really don't know
		5 + (self.0 >= 16) as u8
	}
}

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
	Pause                (bool),
	CreateStringTable    (CreateStringTable),
	UpdateStringTable    (UpdateStringTable),
	VoiceInit            (VoiceInit),
	VoiceData            (VoiceData),
	HltvControl,         // TODO: Not implemented in current protocol version
	PlaySound            (PlaySound),
	SetEntityView        (EntityId),
	FixAngle             (FixAngle),
	CrosshairAngle       (CrosshairAngle),
	Decal                (Decal),
	TerrainMod,          // TODO: Not implemented in current protocol version
	UserMessage          (UserMessage),
	EntityMessage        (EntityMessage),
	GameEvent            (GameEvent),
	Entities             (Entities),
	TempEntities         (TempEntities),
	Prefetch             (Prefetch),
	PluginMenu           (PluginMenu),
	GameEventList        (game_events::GameEventList),
	GetCvar              // TODO: { cookie: u32, key: String }
}

impl Packet {
	pub fn kind(&self) -> PacketKind {
		match *self {
			Packet::Nop => PacketKind::Nop,
			Packet::Disconnect => PacketKind::Disconnect,
			Packet::TransferFile(_) => PacketKind::TransferFile,
			Packet::Tick(_) => PacketKind::Tick,
			Packet::StringCommand(_) => PacketKind::StringCommand,
			Packet::SetCvars(_) => PacketKind::SetCvars,
			Packet::SignonState(_) => PacketKind::SignonState,
			Packet::Print(_) => PacketKind::Print,
			Packet::ServerInfo(_) => PacketKind::ServerInfo,
			Packet::DataTable => PacketKind::DataTable,
			Packet::ClassInfo(_) => PacketKind::ClassInfo,
			Packet::Pause(_) => PacketKind::Pause,
			Packet::CreateStringTable(_) => PacketKind::CreateStringTable,
			Packet::UpdateStringTable(_) => PacketKind::UpdateStringTable,
			Packet::VoiceInit(_) => PacketKind::VoiceInit,
			Packet::VoiceData(_) => PacketKind::VoiceData,
			Packet::HltvControl => PacketKind::HltvControl,
			Packet::PlaySound(_) => PacketKind::PlaySound,
			Packet::SetEntityView(_) => PacketKind::SetEntityView,
			Packet::FixAngle(_) => PacketKind::FixAngle,
			Packet::CrosshairAngle(_) => PacketKind::CrosshairAngle,
			Packet::Decal(_) => PacketKind::Decal,
			Packet::TerrainMod => PacketKind::TerrainMod,
			Packet::UserMessage(_) => PacketKind::UserMessage,
			Packet::EntityMessage(_) => PacketKind::EntityMessage,
			Packet::GameEvent(_) => PacketKind::GameEvent,
			Packet::Entities(_) => PacketKind::Entities,
			Packet::TempEntities(_) => PacketKind::TempEntities,
			Packet::Prefetch(_) => PacketKind::Prefetch,
			Packet::PluginMenu(_) => PacketKind::PluginMenu,
			Packet::GameEventList(_) => PacketKind::GameEventList,
			Packet::GetCvar => PacketKind::GetCvar
		}
	}

	pub fn parse_with_kind(bits: &mut BitReader, kind: PacketKind, version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(match kind {
			PacketKind::Nop               => Packet::Nop,
			PacketKind::Disconnect        => unimplemented!(),
			PacketKind::TransferFile      => Packet::TransferFile     (TransferFile::parse(bits, version)?),
			PacketKind::Tick              => Packet::Tick             (Tick::parse(bits, version)?),
			PacketKind::StringCommand     => Packet::StringCommand    (bits.read_string()?),
			PacketKind::SetCvars          => Packet::SetCvars         (SetCvars::parse(bits, version)?),
			PacketKind::SignonState       => Packet::SignonState      (SignonState::parse(bits, version)?),
			PacketKind::Print             => Packet::Print            (bits.read_string()?),
			PacketKind::ServerInfo        => Packet::ServerInfo       (ServerInfo::parse(bits, version)?),
			PacketKind::DataTable         => unimplemented!(),
			PacketKind::ClassInfo         => Packet::ClassInfo        (ClassInfo::parse(bits, version)?),
			PacketKind::Pause             => Packet::Pause            (bits.read_bit()?),
			PacketKind::CreateStringTable => Packet::CreateStringTable(CreateStringTable::parse(bits, version)?),
			PacketKind::UpdateStringTable => Packet::UpdateStringTable(UpdateStringTable::parse(bits, version)?),
			PacketKind::VoiceInit         => Packet::VoiceInit        (VoiceInit::parse(bits, version)?),
			PacketKind::VoiceData         => Packet::VoiceData        (VoiceData::parse(bits, version)?),
			PacketKind::HltvControl       => unimplemented!(),
			PacketKind::PlaySound         => Packet::PlaySound        (PlaySound::parse(bits, version)?),
			PacketKind::SetEntityView     => Packet::SetEntityView    (bits.read_bits(11)? as u16),
			PacketKind::FixAngle          => Packet::FixAngle         (FixAngle::parse(bits, version)?),
			PacketKind::CrosshairAngle    => Packet::CrosshairAngle   (CrosshairAngle::parse(bits, version)?),
			PacketKind::Decal             => Packet::Decal            (Decal::parse(bits, version)?),
			PacketKind::TerrainMod        => unimplemented!(),
			PacketKind::UserMessage       => Packet::UserMessage      (UserMessage::parse(bits, version)?),
			PacketKind::EntityMessage     => Packet::EntityMessage    (EntityMessage::parse(bits, version)?),
			PacketKind::GameEvent         => Packet::GameEvent        (GameEvent::parse(bits, version)?),
			PacketKind::Entities          => Packet::Entities         (Entities::parse(bits, version)?),
			PacketKind::TempEntities      => Packet::TempEntities     (TempEntities::parse(bits, version)?),
			PacketKind::Prefetch          => Packet::Prefetch         (Prefetch::parse(bits, version)?),
			PacketKind::PluginMenu        => Packet::PluginMenu       (PluginMenu::parse(bits, version)?),
			PacketKind::GameEventList     => Packet::GameEventList    (game_events::GameEventList::parse(bits)?),
			PacketKind::GetCvar           => unimplemented!()
		})
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
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(TransferFile {
			transfer_id: bits.read_u32()?,
			name: bits.read_string()?,
			request_or_deny: bits.read_bit()?
		})
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
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(Tick {
			number: bits.read_u32()?,
			fixed_time: bits.read_u16()?,
			fixed_time_stdev: bits.read_u16()?
		})
	}
}

#[derive(Debug, Clone)]
pub struct SetCvars(pub Vec<(String, String)>);

impl SetCvars {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		let count = bits.read_u8()?;
		let mut cvars = Vec::new();

		for _ in 0..count {
			cvars.push((bits.read_string()?, bits.read_string()?));
		}

		Ok(SetCvars(cvars))
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
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(SignonState {
			state: SignonStateKind::from_id(bits.read_u8()?),
			server_count: bits.read_u32()?
		})
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
	// MapCRC in older versions
	pub _unknown0: Result<[u8; 16], u32>,
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
	pub fn parse(bits: &mut BitReader, version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(ServerInfo {
			network_protocol: bits.read_u16()?,
			server_count: bits.read_u32()?,
			hltv: bits.read_bit()?,
			dedicated: bits.read_bit()?,
			client_dll_crc: bits.read_u32()?,
			max_classes: bits.read_u16()?,
			_unknown0: if version.0 >= 16 { Ok([
				bits.read_u8()?, bits.read_u8()?, bits.read_u8()?, bits.read_u8()?,
				bits.read_u8()?, bits.read_u8()?, bits.read_u8()?, bits.read_u8()?,
				bits.read_u8()?, bits.read_u8()?, bits.read_u8()?, bits.read_u8()?,
				bits.read_u8()?, bits.read_u8()?, bits.read_u8()?, bits.read_u8()?
			]) } else {
				Err(bits.read_u32()?)
			},
			slot: bits.read_u8()?,
			max_clients: bits.read_u8()?,
			tick_interval: bits.read_f32()?,
			os: bits.read_u8()?,
			game_directory: bits.read_string()?,
			map: bits.read_string()?,
			sky: bits.read_string()?,
			hostname: bits.read_string()?,
			_unknown1: if version.0 >= 16 { bits.read_bit()? } else { false }
		})
	}
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
	pub classes: u16,
	pub info: Option<()>
}

impl ClassInfo {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		let classes = bits.read_u16()?;
		let no_parse = bits.read_bit()?;

		if !no_parse {
			unimplemented!("Don't know how to parse the body of ClassInfo!")
		}

		Ok(ClassInfo {
			classes,
			info: None
		})
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
	pub fn parse(bits: &mut BitReader, version: ProtocolVersion) -> Result<Self, ParseError> {
		let name = bits.read_string()?;
		let max_entries = bits.read_u16()?;

		if max_entries == 0 {
			unimplemented!("Don't know how to handle a string table with a maximum of 0 entries")
		}

		let index_bits = (16 - max_entries.leading_zeros()) as u8 - 1;
		let entries = bits.read_bits(index_bits + 1)? as u16;
		let bits_len = if version.0 >= 24 { bits.read_var_u32()? } else { bits.read_bits(20)? };

		// Size and Bits Size
		let fixed_userdata_size = if bits.read_bit()?  {
			Some((bits.read_bits(12)? as u16, bits.read_bits(4)? as u8))
		} else {
			None
		};

		let compressed = bits.read_bit()?;

		let data = Bits::copy_into(bits, bits_len as usize)?;

		Ok(CreateStringTable { name, max_entries, entries, fixed_userdata_size, compressed, data })
	}
}

#[derive(Debug, Clone)]
pub struct UpdateStringTable {
	pub table_id: u8,
	pub entries:  u16,
	pub data:     Bits
}

impl UpdateStringTable {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(UpdateStringTable {
			table_id: bits.read_bits(5)? as u8,
			entries: if bits.read_bit()? { bits.read_u16()? } else { 1 },
			data: {
				let bits_len = bits.read_bits(20)? as usize;
				Bits::copy_into(bits, bits_len)?
			}
		})
	}
}

#[derive(Debug, Clone)]
pub struct VoiceInit {
	pub codec: String,
	pub settings: VoiceSettings
}

impl VoiceInit {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(VoiceInit {
			codec:   bits.read_string()?,
			settings: match bits.read_u8()? {
				255     => VoiceSettings::Extra  (bits.read_u16()?),
				quality => VoiceSettings::Quality(quality)
			}
		})
	}
}

#[derive(Debug, Clone)]
pub enum VoiceSettings {
	Quality(u8),
	Extra(u16)
}

#[derive(Debug, Clone)]
pub struct VoiceData {
	pub sender:    u8,
	pub proximity: u8,
	pub data:      Bits
}

impl VoiceData {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(VoiceData {
			sender: bits.read_u8()?,
			proximity: bits.read_u8()?,
			data: {
				let bits_len = bits.read_u16()?;

				Bits::copy_into(bits, bits_len as usize)?
			}
		})
	}
}

#[derive(Debug, Clone)]
pub enum PlaySound {
	Reliable   (Bits),
	Unreliable { sounds: u8, all: Bits }
}

impl PlaySound {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		let reliable = bits.read_bit()?;

		Ok(if reliable {
			let bits_len = bits.read_u8()?;

			PlaySound::Reliable(Bits::copy_into(bits, bits_len as usize)?)
		} else {
			let sounds = bits.read_u8()?;
			let bits_len = bits.read_u16()?;

			PlaySound::Unreliable { sounds, all: Bits::copy_into(bits, bits_len as usize)? }
		})
	}
}

#[derive(Debug, Clone)]
pub struct FixAngle {
	relative: bool,
	angles: (u16, u16, u16)
}

impl FixAngle {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(FixAngle {
			relative: bits.read_bit()?,
			angles: (
				bits.read_u16()?,
				bits.read_u16()?,
				bits.read_u16()?
			)
		})
	}
}

#[derive(Debug, Clone)]
pub struct CrosshairAngle {
	angles: (u16, u16, u16)
}

impl CrosshairAngle {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(CrosshairAngle {
			angles: (
				bits.read_u16()?,
				bits.read_u16()?,
				bits.read_u16()?
			)
		})
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
	pub fn parse(bits: &mut BitReader, version: ProtocolVersion) -> Result<Self, ParseError> {
		// TODO: More accurate version check!
		// This appears to be related to the size of the `modelprecache` string table.
		let model_index_bits = 12 + (version.0 >= 24) as u8;

		let position = bits.read_vec3()?;
		let decal_index = bits.read_bits(9)? as u16;

		let (entity_index, model_index) = if bits.read_bit()? {
			(bits.read_bits(11)? as u16, bits.read_bits(model_index_bits)? as u16)
		} else {
			(0, 0)
		};

		let low_priority = bits.read_bit()?;

		Ok(Decal { position, decal_index, entity_index, model_index, low_priority })
	}
}

#[derive(Debug, Clone)]
pub struct UserMessage {
	pub channel: u8,
	pub data:    Bits
}

impl UserMessage {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(UserMessage {
			channel: bits.read_u8()?,
			data: {
				let bits_len = bits.read_bits(11)? as usize;
				Bits::copy_into(bits, bits_len)?
			}
		})
	}
}

#[derive(Debug, Clone)]
pub struct EntityMessage {
	pub entity: EntityId,
	pub class:  u16,
	pub data:   Bits
}

impl EntityMessage {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(EntityMessage {
			entity: bits.read_bits(11)? as u16,
			class:  bits.read_bits(9)? as u16,
			data: {
				let bits_len = bits.read_bits(11)? as usize;
				Bits::copy_into(bits, bits_len)?
			}
		})
	}
}

// First 9 bits are the event ID
#[derive(Debug, Clone)]
pub struct GameEvent(pub Bits);

impl GameEvent {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		let bits_len = bits.read_bits(11)? as usize;

		Ok(GameEvent(Bits::copy_into(bits, bits_len)?))
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
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		let max_entries = bits.read_bits(11)? as u16;

		let delta_from_tick = if bits.read_bit()? {
			Some(bits.read_u32()?)
		} else {
			None
		};

		let baseline = bits.read_bit()?;
		let updated = bits.read_bits(11)? as u16;
		let bits_len = bits.read_bits(20)? as usize;
		let update_baseline = bits.read_bit()?;

		Ok(Entities {
			max_entries,
			delta_from_tick,
			baseline,
			updated,
			update_baseline,
			data: Bits::copy_into(bits, bits_len)?
		})
	}
}

pub struct TempEntities {
	pub count: u8,
	pub data:  Bits
}

impl TempEntities {
	pub fn parse(bits: &mut BitReader, version: ProtocolVersion) -> Result<Self, ParseError> {
		let count = bits.read_u8()?;
		let bits_len = if version.0 >= 24 { bits.read_var_u32()? } else {bits.read_bits(17)? };

		Ok(TempEntities {
			count,
			data: Bits::copy_into(bits, bits_len as usize)?
		})
	}
}

#[derive(Debug, Clone)]
pub struct Prefetch {
	pub kind: bool,
	pub id: u16
}

impl Prefetch {
	pub fn parse(bits: &mut BitReader, version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(Prefetch {
			kind: if version.0 >= 23 { bits.read_bit()? } else { false },
			id:   bits.read_bits(13)? as u16
		})
	}
}

#[derive(Debug, Clone)]
pub struct PluginMenu {
	pub kind: u16,
	/// KeyValues encoded into a byte buffer
	pub data: Vec<u8>
}

impl PluginMenu {
	pub fn parse(bits: &mut BitReader, _version: ProtocolVersion) -> Result<Self, ParseError> {
		Ok(PluginMenu {
			kind: bits.read_u16()?,
			data: {
				let length = bits.read_u16()?;

				bits.read_u8_array(length as usize)?
			}
		})
	}
}