mod convert;
mod db;
mod de;
mod ser;

use std::collections::BTreeMap;

use crate::db::custom_types::date_str::{DateStr, DateTimeStr};
use crate::error::{server_err, server_err_owned, CoreErrorCodes};
use crate::input_helper::{json_to_string, qs_to_string};
use crate::res::AppRes;

#[derive(Debug)]
pub enum Value
{
	Null,
	Bool(bool),
	Int(i64),
	UInt(u64),
	Float(f64),
	String(String),
	Bytes(Vec<u8>),
	Date(DateStr),         //Will not be deserialized but later set from string
	DateTime(DateTimeStr), //Will not be deserialized but later set from string
	Array(Vec<Value>),
	Object(BTreeMap<String, Value>),
}

impl Value
{
	pub fn to_json(&self) -> AppRes<String>
	{
		json_to_string(self)
	}

	pub fn to_qs(&self) -> AppRes<String>
	{
		qs_to_string(self)
	}

	pub fn transform_to_date(self) -> AppRes<Self>
	{
		let out = match self {
			Value::Date(_) => self,
			Value::DateTime(d) => Value::Date(d.into()),
			Value::String(s) => Value::Date(s.parse()?),
			_ => {
				return Err(server_err(
					400,
					CoreErrorCodes::JsonParse,
					"Type can't cast to Date. Only Date, Datetime and String are supported",
				))
			},
		};

		Ok(out)
	}

	pub fn transform_to_datetime(self) -> AppRes<Self>
	{
		let out = match self {
			Value::Date(d) => Value::DateTime(d.into()),
			Value::DateTime(_) => self,
			Value::String(s) => Value::DateTime(s.parse()?),
			_ => {
				return Err(server_err(
					400,
					CoreErrorCodes::JsonParse,
					"Type can't cast to Datetime. Only Date, Datetime and String are supported",
				))
			},
		};

		Ok(out)
	}

	pub fn str_to_date(str: &str) -> AppRes<Self>
	{
		Ok(Value::Date(str.parse()?))
	}

	pub fn str_to_date_time(str: &str) -> AppRes<Self>
	{
		Ok(Value::DateTime(str.parse()?))
	}

	pub fn from_json(str: &str) -> AppRes<Self>
	{
		serde_json::from_str(str).map_err(|e| server_err_owned(422, CoreErrorCodes::JsonParse, format!("Wrong input: {:?}", e), None))
	}

	pub fn from_qs(str: &str) -> AppRes<Self>
	{
		let map: ValueMap =
			serde_qs::from_str(str).map_err(|e| server_err_owned(422, CoreErrorCodes::JsonParse, format!("Wrong input: {:?}", e), None))?;

		Ok(map.into())
	}
}

/**
Same as value but deserializing a mpa instead of any.

This is used when the Deserializer not support any.
 */
pub struct ValueMap(Value);

#[allow(clippy::from_over_into)]
impl Into<Value> for ValueMap
{
	fn into(self) -> Value
	{
		self.0
	}
}

/**
After fetching from sql put the columns of each row into this inner array.
*/
#[derive(Debug)]
pub struct OutputRow(DbRow);

pub type DbRow = Vec<Value>;

#[allow(clippy::from_over_into)]
impl Into<DbRow> for OutputRow
{
	fn into(self) -> DbRow
	{
		self.0
	}
}
