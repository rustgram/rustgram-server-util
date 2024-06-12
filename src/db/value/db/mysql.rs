use crate::db::custom_types::date_str::DateTimeStr;
use crate::db::mysql_async_export::prelude::{FromRow, FromValue};
use crate::db::mysql_async_export::{FromRowError, FromValueError, Row, Value as MysqlValue};
use crate::db::value::{OutputRow, Value};

impl FromValue for Value
{
	type Intermediate = Value;
}

impl TryFrom<MysqlValue> for Value
{
	type Error = FromValueError;

	fn try_from(value: MysqlValue) -> Result<Self, Self::Error>
	{
		let out = match value {
			MysqlValue::NULL => Self::Null,
			MysqlValue::Bytes(ref _b) => Self::String(String::from_value_opt(value)?),
			MysqlValue::Int(i) => i.into(),
			MysqlValue::UInt(u) => u.into(),
			MysqlValue::Float(f) => f.into(),
			MysqlValue::Double(f) => f.into(),
			MysqlValue::Date(_, _, _, _, _, _, _) => Self::DateTime(DateTimeStr::from_value_opt(value)?),
			MysqlValue::Time(_, _, _, _, _, _) => return Err(FromValueError(value)),
		};

		Ok(out)
	}
}

#[allow(clippy::from_over_into)]
impl Into<MysqlValue> for Value
{
	fn into(self) -> MysqlValue
	{
		match self {
			Value::Null => MysqlValue::NULL,
			Value::Bool(b) => b.into(),
			Value::Int(i) => i.into(),
			Value::UInt(u) => u.into(),
			Value::Float(f) => f.into(),
			Value::String(s) => s.into(),
			Value::Bytes(b) => MysqlValue::Bytes(b),
			Value::Date(d) => d.into(),
			Value::DateTime(d) => d.into(),
			//array and obj only for input out not internal
			Value::Array(_) => MysqlValue::NULL,
			Value::Object(_) => MysqlValue::NULL,
		}
	}
}

fn get_next_row(row: &mut Row, i: usize) -> Result<Option<Value>, FromRowError>
{
	let out = row.take_opt(i);

	if let Some(o) = out {
		Ok(Some(o.map_err(|_| FromRowError(row.clone()))?))
	} else {
		Ok(None)
	}
}

impl FromRow for OutputRow
{
	fn from_row_opt(mut row: Row) -> Result<Self, FromRowError>
	where
		Self: Sized,
	{
		let mut vec = vec![];
		let mut index = 0;

		while let Some(v) = get_next_row(&mut row, index)? {
			vec.push(v);
			index += 1;
		}

		Ok(OutputRow(vec))
	}
}
