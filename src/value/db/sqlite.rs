use crate::db::rusqlite_export::types::{FromSql, FromSqlResult, Value as SqliteValue, ValueRef};
use crate::db::rusqlite_export::{Error as SqliteError, Row};
use crate::db::{FormSqliteRowError, FromSqliteRow};
use crate::value::{OutputRow, Value};

impl FromSql for Value
{
	fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self>
	{
		let out = match value {
			ValueRef::Null => Self::Null,
			ValueRef::Integer(i) => i.into(),
			ValueRef::Real(f) => f.into(),
			ValueRef::Text(_) => Self::String(String::column_result(value)?),
			ValueRef::Blob(b) => Self::Bytes(b.to_owned()),
		};

		Ok(out)
	}
}

#[allow(clippy::from_over_into)]
impl Into<SqliteValue> for Value
{
	fn into(self) -> SqliteValue
	{
		match self {
			Value::Null => SqliteValue::Null,
			Value::Bool(b) => b.into(),
			Value::Int(i) => i.into(),
			Value::UInt(u) => SqliteValue::Integer(u as i64),
			Value::Float(f) => f.into(),
			Value::String(s) => s.into(),
			Value::Bytes(b) => b.into(),
			Value::Date(d) => d.into(),
			Value::DateTime(d) => d.into(),
			Value::Array(_) => SqliteValue::Null,
			Value::Object(_) => SqliteValue::Null,
		}
	}
}

fn get_next_row(row: &Row, i: usize) -> Result<Option<Value>, FormSqliteRowError>
{
	let out = row.get(i);

	match out {
		Ok(r) => Ok(Some(r)),
		Err(e) => {
			match e {
				//end of the row columns
				SqliteError::InvalidColumnIndex(_) => Ok(None),
				SqliteError::InvalidColumnName(_) => Ok(None),
				_ => {
					Err(FormSqliteRowError {
						msg: e.to_string(),
					})
				},
			}
		},
	}
}

impl FromSqliteRow for OutputRow
{
	fn from_row_opt(row: &Row) -> Result<Self, FormSqliteRowError>
	where
		Self: Sized,
	{
		let mut vec = vec![];
		let mut index = 0;

		while let Some(v) = get_next_row(row, index)? {
			vec.push(v);
			index += 1;
		}

		Ok(OutputRow(vec))
	}
}
