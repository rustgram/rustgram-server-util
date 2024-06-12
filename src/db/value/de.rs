use std::collections::BTreeMap;
use std::fmt::Formatter;

use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::db::value::{Value, ValueMap};

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor
{
	type Value = Value;

	fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result
	{
		formatter.write_str("any valid JSON value")
	}

	#[inline]
	fn visit_bool<E>(self, value: bool) -> Result<Value, E>
	{
		Ok(Value::Bool(value))
	}

	#[inline]
	fn visit_i64<E>(self, value: i64) -> Result<Value, E>
	{
		Ok(value.into())
	}

	#[inline]
	fn visit_u64<E>(self, value: u64) -> Result<Value, E>
	{
		Ok(value.into())
	}

	#[inline]
	fn visit_f64<E>(self, value: f64) -> Result<Value, E>
	{
		Ok(value.into())
	}

	#[inline]
	fn visit_str<E>(self, value: &str) -> Result<Value, E>
	where
		E: serde::de::Error,
	{
		self.visit_string(String::from(value))
	}

	#[inline]
	fn visit_string<E>(self, value: String) -> Result<Value, E>
	{
		Ok(Value::String(value))
	}

	#[inline]
	fn visit_none<E>(self) -> Result<Value, E>
	{
		Ok(Value::Null)
	}

	#[inline]
	fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		Deserialize::deserialize(deserializer)
	}

	#[inline]
	fn visit_seq<V>(self, mut visitor: V) -> Result<Value, V::Error>
	where
		V: SeqAccess<'de>,
	{
		let mut vec = Vec::new();

		while let Some(elem) = visitor.next_element()? {
			vec.push(elem);
		}

		Ok(Value::Array(vec))
	}

	fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
	where
		V: MapAccess<'de>,
	{
		match visitor.next_key_seed(KeyClassifier)? {
			None => Ok(Value::Object(BTreeMap::new())),
			Some(first_key) => {
				let mut values = BTreeMap::new();

				values.insert(first_key, visitor.next_value()?);
				while let Some((key, value)) = visitor.next_entry()? {
					values.insert(key, value);
				}

				Ok(Value::Object(values))
			},
		}
	}
}

struct KeyClassifier;

impl<'de> DeserializeSeed<'de> for KeyClassifier
{
	type Value = String;

	fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
	where
		D: Deserializer<'de>,
	{
		Deserialize::deserialize(deserializer)
	}
}

impl<'de> Deserialize<'de> for Value
{
	#[inline]
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_any(ValueVisitor)
	}
}

impl<'de> Deserialize<'de> for ValueMap
{
	#[inline]
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		Ok(Self(deserializer.deserialize_map(ValueVisitor)?))
	}
}

#[cfg(test)]
mod test
{
	use std::env;

	use serde_json::{from_str, json};

	use super::*;

	#[test]
	fn test_from_json()
	{
		let v = json!(true);
		let v = v.to_string();

		let v = Value::from_json(&v).unwrap();

		if let Value::Bool(b) = v {
			assert!(b);
		} else {
			panic!("Wrong value for bool")
		}

		let v = json!(12.5);
		let v = v.to_string();

		let v: Value = from_str(&v).unwrap();

		if let Value::Float(n) = v {
			assert_eq!(n, 12.5);
		} else {
			panic!("Wrong value for number")
		}

		let v = json!("a string");
		let v = v.to_string();

		let v: Value = from_str(&v).unwrap();

		if let Value::String(s) = v {
			assert_eq!(s, "a string");
		} else {
			panic!("Wrong value for string")
		}

		let v = json!(["an", "array"]);
		let v = v.to_string();

		let v: Value = from_str(&v).unwrap();

		if let Value::Array(a) = v {
			assert_eq!(
				a.first().unwrap().to_json().unwrap(),
				Value::String("an".to_string()).to_json().unwrap()
			);
			assert_eq!(
				a.get(1).unwrap().to_json().unwrap(),
				Value::String("array".to_string()).to_json().unwrap()
			);
		} else {
			panic!("Wrong value for array")
		}

		let v = json!({ "an": "object" });
		let v = v.to_string();

		let v: Value = from_str(&v).unwrap();

		if let Value::Object(a) = v {
			let v = a.get("an").unwrap();
			assert_eq!(
				v.to_json().unwrap(),
				Value::String("object".to_string()).to_json().unwrap()
			)
		} else {
			panic!("Wrong value for object")
		}

		let v = json!("2024-10-02");
		let v = v.to_string();

		let v: Value = from_str(&v).unwrap();

		if let Value::String(s) = v {
			let v = Value::str_to_date(&s).unwrap();

			if let Value::Date(d) = v {
				assert_eq!(d.year, 2024);
			} else {
				panic!("Wrong value for date")
			}
		} else {
			panic!("Wrong value for date string")
		}

		let v = json!("2024-10-02 02:30:02");
		let v = v.to_string();

		let v: Value = from_str(&v).unwrap();

		if let Value::String(s) = v {
			let v = Value::str_to_date_time(&s).unwrap();

			if let Value::DateTime(d) = v {
				assert_eq!(d.year, 2024);
				assert_eq!(d.hour, 2);
			} else {
				panic!("Wrong value for date time")
			}
		} else {
			panic!("Wrong value for date time string")
		}
	}

	#[test]
	fn test_from_qs()
	{
		let str =
			"name=Acme&id=42&phone=12345&address[postcode]=12345&address[city]=Carrot+City&user_ids[0]=1&user_ids[1]=2&user_ids[2]=3&user_ids[3]=4";

		let v = Value::from_qs(str).unwrap();

		let out = Value::Object(BTreeMap::from([
			(
				"address".to_string(),
				Value::Object(BTreeMap::from([
					("city".to_string(), "Carrot City".into()),
					("postcode".to_string(), "12345".into()),
				])),
			),
			("id".to_string(), "42".into()),
			("name".to_string(), "Acme".into()),
			("phone".to_string(), "12345".into()),
			(
				"user_ids".to_string(),
				Value::Array(vec!["1".into(), "2".into(), "3".into(), "4".into()]),
			),
		]));

		assert_eq!(v.to_json().unwrap(), out.to_json().unwrap());
	}

	#[test]
	fn test_custom_json()
	{
		dotenv::dotenv().ok();

		let str = env::var("json_example_str").unwrap();

		let v = Value::from_json(&str).unwrap();

		println!("{:?}", v);
		println!("{}", v.to_json().unwrap());
	}

	#[test]
	fn test_custom_qs()
	{
		dotenv::dotenv().ok();

		let str = env::var("qs_example_str").unwrap();

		let v = Value::from_qs(&str).unwrap();

		println!("{:?}", v);
		println!("{}", v.to_json().unwrap());
	}
}
