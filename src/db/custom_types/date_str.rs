use std::fmt::Display;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{server_err, CoreErrorCodes, ServerCoreError};

macro_rules! single_time_position_str {
	($p:expr) => {
		if $p < 10 {
			format!("0{}", $p)
		} else {
			format!("{}", $p)
		}
	};
}

pub fn get_date_str_from_number(year: u16, month: u8, day: u8) -> String
{
	//adding the leading zero

	let day = if day < 10 { format!("0{day}") } else { format!("{day}") };
	let month = if month < 10 { format!("0{month}") } else { format!("{month}") };

	format!("{}-{}-{}", year, month, day)
}

pub fn get_time_str_from_number(hour: u8, min: u8, sec: u8) -> (String, String, String)
{
	let hour = single_time_position_str!(hour);
	let min = single_time_position_str!(min);
	let sec = single_time_position_str!(sec);

	(hour, min, sec)
}

pub fn get_milliseconds_str_from_number(milli: u32) -> String
{
	if milli < 10 {
		format!("00{milli}")
	} else if milli < 100 {
		format!("0{milli}")
	} else {
		format!("{milli}")
	}
}

//__________________________________________________________________________________________________

type DateFormatParts = (u16, u8, u8);
type DateTimeFormatParts = (u16, u8, u8, u8, u8, u8);
type DateTimeFormatMilliSecParts = (u16, u8, u8, u8, u8, u8, u32);

fn get_date_format_parts(date_str: &str) -> Option<DateFormatParts>
{
	// Check if the length of the string is exactly 10 characters
	if date_str.len() != 10 {
		return None;
	}

	// Split the string into parts separated by '-'
	let parts: Vec<&str> = date_str.split('-').collect();

	// Ensure we have exactly three parts (year, month, day)
	if parts.len() != 3 {
		return None;
	}

	// Ensure each part is convertible to a u32
	let year: Option<u16> = parts[0].parse().ok();
	let month: Option<u8> = parts[1].parse().ok();
	let day: Option<u8> = parts[2].parse().ok();

	// Check if all parts are valid numbers
	if year.is_none() || month.is_none() || day.is_none() {
		return None;
	}

	// Check if year, month, and day are in valid ranges
	let year = year.unwrap();
	let month = month.unwrap();
	let day = day.unwrap();

	#[allow(clippy::manual_range_contains)]
	if year < 1000 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31 {
		return None;
	}

	Some((year, month, day))
}

fn get_date_time_format_parts(date_time_str: &str) -> Option<DateTimeFormatParts>
{
	// Check if the length of the string is exactly 19 characters
	if date_time_str.len() != 19 {
		return None;
	}

	// Split the string into parts separated by '-', ' ', and ':'
	let parts: Vec<&str> = date_time_str.split(&['-', ' ', ':'][..]).collect();

	// Ensure we have exactly six parts (year, month, day, hour, minute, second)
	if parts.len() != 6 {
		return None;
	}

	// Ensure each part is convertible to a u32
	let year: Option<u16> = parts[0].parse().ok();
	let month: Option<u8> = parts[1].parse().ok();
	let day: Option<u8> = parts[2].parse().ok();
	let hour: Option<u8> = parts[3].parse().ok();
	let minute: Option<u8> = parts[4].parse().ok();
	let second: Option<u8> = parts[5].parse().ok();

	// Check if all parts are valid numbers
	if year.is_none() || month.is_none() || day.is_none() || hour.is_none() || minute.is_none() || second.is_none() {
		return None;
	}

	// Check if year, month, day, hour, minute, and second are in valid ranges
	let year = year.unwrap();
	let month = month.unwrap();
	let day = day.unwrap();
	let hour = hour.unwrap();
	let minute = minute.unwrap();
	let second = second.unwrap();

	#[allow(clippy::manual_range_contains)]
	if year < 1000 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31 || hour > 23 || minute > 59 || second > 59 {
		return None;
	}

	Some((year, month, day, hour, minute, second))
}

fn get_date_time_format_milli_sec_parts(datetime_str: &str) -> Option<DateTimeFormatMilliSecParts>
{
	// Check if the length of the string is exactly 23 characters
	if datetime_str.len() != 23 {
		return None;
	}

	// Split the string into parts separated by '-', ' ', ':', and '.'
	let parts: Vec<&str> = datetime_str.split(&['-', ' ', ':', '.'][..]).collect();

	// Ensure we have exactly seven parts (year, month, day, hour, minute, second, millisecond)
	if parts.len() != 7 {
		return None;
	}

	// Ensure each part is convertible to a u32
	let year: Option<u16> = parts[0].parse().ok();
	let month: Option<u8> = parts[1].parse().ok();
	let day: Option<u8> = parts[2].parse().ok();
	let hour: Option<u8> = parts[3].parse().ok();
	let minute: Option<u8> = parts[4].parse().ok();
	let second: Option<u8> = parts[5].parse().ok();
	let millisecond: Option<u32> = parts[6].parse().ok();

	// Check if all parts are valid numbers
	if year.is_none() || month.is_none() || day.is_none() || hour.is_none() || minute.is_none() || second.is_none() || millisecond.is_none() {
		return None;
	}

	// Check if year, month, day, hour, minute, second, and millisecond are in valid ranges
	let year = year.unwrap();
	let month = month.unwrap();
	let day = day.unwrap();
	let hour = hour.unwrap();
	let minute = minute.unwrap();
	let second = second.unwrap();
	let millisecond = millisecond.unwrap();

	#[allow(clippy::manual_range_contains)]
	if year < 1000 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31 || hour > 23 || minute > 59 || second > 59 || millisecond > 999 {
		return None;
	}

	Some((year, month, day, hour, minute, second, millisecond))
}

//__________________________________________________________________________________________________

macro_rules! sqlite_from_str (
	($t:ty) => (
		#[cfg(feature = "sqlite")]
		impl rusqlite::types::FromSql for $t
		{
			fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self>
			{
				value.as_str().and_then(|s| {
					match s.parse() {
						Ok(s) => Ok(s),
						Err(_e) => Err(rusqlite::types::FromSqlError::InvalidType),
					}
				})
			}
		}
	)
);

macro_rules! serialize_to_str (
	($t:ty) => (
		impl Serialize for $t
		{
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: Serializer,
			{
				serializer.serialize_str(&self.to_string())
			}
		}
	)
);

macro_rules! deserialize_from_str (
	($t:ty) => (
		impl<'de> Deserialize<'de> for $t
		{
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: Deserializer<'de>,
			{
				let s: String = Deserialize::deserialize(deserializer)?;

				match s.parse() {
					Ok(s) => Ok(s),
					Err(e) => Err(de::Error::custom(e.msg)),
				}
			}
		}
	)
);

#[derive(Debug)]
pub struct DateStr
{
	pub year: u16,
	pub month: u8,
	pub day: u8,
}

impl FromStr for DateStr
{
	type Err = ServerCoreError;

	fn from_str(s: &str) -> Result<Self, Self::Err>
	{
		if let Some((year, month, day)) = get_date_format_parts(s) {
			Ok(Self {
				year,
				month,
				day,
			})
		} else {
			Err(server_err(
				400,
				CoreErrorCodes::DateStrParse,
				"Date is in a wrong format. Accepted format: YYY-MM-DD",
			))
		}
	}
}

impl Display for DateStr
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "{}", get_date_str_from_number(self.year, self.month, self.day))
	}
}

serialize_to_str!(DateStr);
deserialize_from_str!(DateStr);
sqlite_from_str!(DateStr);

#[cfg(feature = "mysql")]
impl mysql_common::prelude::FromValue for DateStr
{
	type Intermediate = DateStr;
}

#[cfg(feature = "mysql")]
impl TryFrom<mysql_common::Value> for DateStr
{
	type Error = mysql_common::FromValueError;

	fn try_from(value: mysql_common::Value) -> Result<Self, Self::Error>
	{
		match value {
			//ignore the time for only dates
			mysql_common::Value::Date(year, month, day, _, _, _, _) => {
				Ok(Self {
					year,
					month,
					day,
				})
			},
			_ => Err(mysql_common::FromValueError(value)),
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Debug)]
pub struct DateTimeStr
{
	pub year: u16,
	pub month: u8,
	pub day: u8,
	pub hour: u8,
	pub minute: u8,
	pub second: u8,
}

impl FromStr for DateTimeStr
{
	type Err = ServerCoreError;

	fn from_str(s: &str) -> Result<Self, Self::Err>
	{
		if let Some((year, month, day, hour, minute, second)) = get_date_time_format_parts(s) {
			Ok(Self {
				year,
				month,
				day,
				hour,
				minute,
				second,
			})
		} else {
			Err(server_err(
				400,
				CoreErrorCodes::DateStrParse,
				"Date time is in a wrong format. Accepted format: YYY-MM-DD HH:MM:SS",
			))
		}
	}
}

impl Display for DateTimeStr
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		let (hour, min, sec) = get_time_str_from_number(self.hour, self.minute, self.second);

		write!(
			f,
			"{} {}:{}:{}",
			get_date_str_from_number(self.year, self.month, self.day),
			hour,
			min,
			sec
		)
	}
}

serialize_to_str!(DateTimeStr);
deserialize_from_str!(DateTimeStr);
sqlite_from_str!(DateTimeStr);

#[cfg(feature = "mysql")]
impl mysql_common::prelude::FromValue for DateTimeStr
{
	type Intermediate = DateTimeStr;
}

#[cfg(feature = "mysql")]
impl TryFrom<mysql_common::Value> for DateTimeStr
{
	type Error = mysql_common::FromValueError;

	fn try_from(value: mysql_common::Value) -> Result<Self, Self::Error>
	{
		match value {
			//ignore the time for only dates
			mysql_common::Value::Date(year, month, day, hour, minute, second, _) => {
				Ok(Self {
					year,
					month,
					day,
					hour,
					minute,
					second,
				})
			},
			_ => Err(mysql_common::FromValueError(value)),
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Debug)]
pub struct DateTimeMilliStr
{
	pub year: u16,
	pub month: u8,
	pub day: u8,
	pub hour: u8,
	pub minute: u8,
	pub second: u8,
	pub milli_seconds: u32,
}

impl FromStr for DateTimeMilliStr
{
	type Err = ServerCoreError;

	fn from_str(s: &str) -> Result<Self, Self::Err>
	{
		if let Some((year, month, day, hour, minute, second, milli_seconds)) = get_date_time_format_milli_sec_parts(s) {
			Ok(Self {
				year,
				month,
				day,
				hour,
				minute,
				second,
				milli_seconds,
			})
		} else {
			Err(server_err(
				400,
				CoreErrorCodes::DateStrParse,
				"Date time is in a wrong format. Accepted format: YYY-MM-DD HH:MM:SS.MMM",
			))
		}
	}
}

impl Display for DateTimeMilliStr
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
	{
		let (hour, min, sec) = get_time_str_from_number(self.hour, self.minute, self.second);
		let milli = get_milliseconds_str_from_number(self.milli_seconds);

		write!(
			f,
			"{} {}:{}:{}.{}",
			get_date_str_from_number(self.year, self.month, self.day),
			hour,
			min,
			sec,
			milli
		)
	}
}

serialize_to_str!(DateTimeMilliStr);
deserialize_from_str!(DateTimeMilliStr);
sqlite_from_str!(DateTimeMilliStr);

#[cfg(feature = "mysql")]
impl mysql_common::prelude::FromValue for DateTimeMilliStr
{
	type Intermediate = DateTimeMilliStr;
}

#[cfg(feature = "mysql")]
impl TryFrom<mysql_common::Value> for DateTimeMilliStr
{
	type Error = mysql_common::FromValueError;

	fn try_from(value: mysql_common::Value) -> Result<Self, Self::Error>
	{
		match value {
			//ignore the time for only dates
			mysql_common::Value::Date(year, month, day, hour, minute, second, milli_seconds) => {
				Ok(Self {
					year,
					month,
					day,
					hour,
					minute,
					second,
					milli_seconds: milli_seconds / 1000, //returns micro sec
				})
			},
			_ => Err(mysql_common::FromValueError(value)),
		}
	}
}

//__________________________________________________________________________________________________

#[derive(Debug)]
pub struct TimeSinglePositionStr(pub String);

impl Serialize for TimeSinglePositionStr
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.0)
	}
}

#[cfg(feature = "mysql")]
impl mysql_common::prelude::FromValue for TimeSinglePositionStr
{
	type Intermediate = TimeSinglePositionStr;
}

#[cfg(feature = "mysql")]
impl TryFrom<mysql_common::Value> for TimeSinglePositionStr
{
	type Error = mysql_common::FromValueError;

	fn try_from(value: mysql_common::Value) -> Result<Self, Self::Error>
	{
		match value {
			mysql_common::Value::UInt(p) => Ok(Self(single_time_position_str!(p))),
			mysql_common::Value::Int(p) => Ok(Self(single_time_position_str!(p))),
			_ => Err(mysql_common::FromValueError(value)),
		}
	}
}

#[cfg(feature = "sqlite")]
impl rusqlite::types::FromSql for TimeSinglePositionStr
{
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self>
	{
		match value {
			rusqlite::types::ValueRef::Integer(p) => Ok(Self(single_time_position_str!(p))),
			_ => Err(rusqlite::types::FromSqlError::InvalidType),
		}
	}
}
