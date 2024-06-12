use crate::db::value::Value;

impl From<i64> for Value
{
	fn from(value: i64) -> Self
	{
		Self::Int(value)
	}
}

impl From<i32> for Value
{
	fn from(value: i32) -> Self
	{
		Self::Int(value as i64)
	}
}

impl From<u64> for Value
{
	fn from(value: u64) -> Self
	{
		Self::UInt(value)
	}
}

impl From<u32> for Value
{
	fn from(value: u32) -> Self
	{
		Self::UInt(value as u64)
	}
}

impl From<f64> for Value
{
	fn from(value: f64) -> Self
	{
		Self::Float(value)
	}
}

impl From<f32> for Value
{
	fn from(value: f32) -> Self
	{
		Self::Float(value as f64)
	}
}

impl From<String> for Value
{
	fn from(value: String) -> Self
	{
		Self::String(value)
	}
}

impl<'a> From<&'a str> for Value
{
	fn from(value: &'a str) -> Self
	{
		Self::String(value.to_string())
	}
}
