use serde::Serialize;

use crate::value::Value;

impl Serialize for Value
{
	#[inline]
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self {
			Self::Null => serializer.serialize_unit(),
			Self::Bool(b) => serializer.serialize_bool(*b),
			Self::Int(n) => serializer.serialize_i64(*n),
			Self::UInt(n) => serializer.serialize_u64(*n),
			Self::Float(n) => serializer.serialize_f64(*n),
			Self::Bytes(b) => serializer.serialize_bytes(b),
			Self::String(s) => serializer.serialize_str(s),
			Self::Date(d) => d.serialize(serializer),
			Self::DateTime(d) => d.serialize(serializer),
			Self::Array(v) => v.serialize(serializer),
			Self::Object(m) => {
				use serde::ser::SerializeMap;
				let mut map = serializer.serialize_map(Some(m.len()))?;
				for (k, v) in m {
					map.serialize_entry(k, v)?;
				}
				map.end()
			},
		}
	}
}

#[cfg(test)]
mod test
{
	use std::collections::BTreeMap;

	use super::*;
	use crate::db::custom_types::date_str::{DateStr, DateTimeStr};

	#[test]
	fn to_json()
	{
		let v = Value::Bool(true);
		assert_eq!(v.to_json().unwrap(), "true");

		let v = Value::Float(12.5);
		assert_eq!(v.to_json().unwrap(), "12.5");

		let v = Value::String("a string".to_string());
		assert_eq!(v.to_json().unwrap(), r#""a string""#);

		let v = Value::Array(vec!["an".into(), "array".into()]);
		assert_eq!(v.to_json().unwrap(), r#"["an","array"]"#);

		let v = Value::Object(BTreeMap::from([("an".to_string(), "object".into())]));
		assert_eq!(v.to_json().unwrap(), r#"{"an":"object"}"#);

		let v = Value::Date(DateStr {
			year: 2024,
			month: 10,
			day: 2,
		});

		assert_eq!(v.to_json().unwrap(), r#""2024-10-02""#);

		let v = Value::DateTime(DateTimeStr {
			year: 2024,
			month: 10,
			day: 2,
			hour: 2,
			minute: 30,
			second: 2,
		});

		assert_eq!(v.to_json().unwrap(), r#""2024-10-02 02:30:02""#);
	}

	#[test]
	fn to_qs()
	{
		let v = Value::Object(BTreeMap::from([("an".to_string(), "object".into())]));

		let str = v.to_qs().unwrap();

		assert_eq!(str, "an=object");

		println!("{str}");
	}
}
