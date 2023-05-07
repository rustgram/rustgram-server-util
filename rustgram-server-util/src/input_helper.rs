use bytes::BytesMut;
use futures::StreamExt;
use rustgram::Request;
use serde::{de, Serialize};
use serde_json::{from_slice, to_string};

use crate::error::{CoreErrorCodes, ServerCoreError, ServerErrorConstructor};

const MAX_SIZE_JSON: usize = 262_144; // max payload size is 256k

pub async fn get_raw_body(req: &mut Request) -> Result<BytesMut, ServerCoreError>
{
	//read the json to memory
	let req_body = req.body_mut();
	let mut body = BytesMut::new();

	while let Some(bytes) = req_body.next().await {
		match bytes {
			Ok(chunk) => {
				if (body.len() + chunk.len()) > MAX_SIZE_JSON {
					return Err(ServerCoreError::new_msg(
						413,
						CoreErrorCodes::InputTooBig,
						"Input was too big to handle",
					));
				}

				body.extend_from_slice(&chunk);
			},
			Err(_) => {
				continue;
			},
		}
	}

	Ok(body)
}

pub fn json_to_string<T>(value: &T) -> Result<String, ServerCoreError>
where
	T: ?Sized + Serialize,
{
	match to_string(value) {
		Ok(o) => Ok(o),
		Err(e) => {
			Err(ServerCoreError::new_msg_owned(
				422,
				CoreErrorCodes::JsonToString,
				format!("json parse err: {:?}", e),
				None,
			))
		},
	}
}

pub fn bytes_to_json<'a, T>(v: &'a [u8]) -> Result<T, ServerCoreError>
where
	T: de::Deserialize<'a>,
{
	match from_slice::<T>(v) {
		Ok(o) => Ok(o),
		Err(e) => {
			Err(ServerCoreError::new_msg_owned(
				422,
				CoreErrorCodes::JsonParse,
				format!("Wrong input: {:?}", e),
				None,
			))
		},
	}
}
