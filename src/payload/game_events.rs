use std::collections::HashMap;

#[derive(Debug)]
pub enum Value {
	Str(String),
	F32(f32),
	I32(i32),
	I16(i16),
	U8(u8),
	Bool(bool)
}

pub struct GameEventData(pub HashMap<String, Value>);

impl GameEventData {
	pub fn get_str(&self, name: &str) -> Option<&str> {
		self.0.get(name).and_then(|value| match value {
			&Value::Str(ref value) => Some(value.as_ref()),
			_ => None
		})
	}

	pub fn get_f32(&self, name: &str) -> Option<f32> {
		self.0.get(name).and_then(|value| match value {
			&Value::F32(value) => Some(value),
			_ => None
		})
	}

	pub fn get_i32(&self, name: &str) -> Option<i32> {
		self.0.get(name).and_then(|value| match value {
			&Value::I32(value) => Some(value),
			_ => None
		})
	}

	pub fn get_i16(&self, name: &str) -> Option<i16> {
		self.0.get(name).and_then(|value| match value {
			&Value::I16(value) => Some(value),
			_ => None
		})
	}

	pub fn get_u8(&self, name: &str) -> Option<u8> {
		self.0.get(name).and_then(|value| match value {
			&Value::U8(value) => Some(value),
			_ => None
		})
	}

	pub fn get_bool(&self, name: &str) -> Option<bool> {
		self.0.get(name).and_then(|value| match value {
			&Value::Bool(value) => Some(value),
			_ => None
		})
	}
}