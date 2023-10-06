use uuid::{Uuid, Version};

use crate::error::{CoreErrorCodes, ServerCoreError, ServerErrorConstructor};
use crate::res::AppRes;

pub fn check_id_format(id: &str) -> AppRes<()>
{
	let uuid = Uuid::try_parse(id).map_err(|_e| {
		ServerCoreError::new_msg(
			400,
			CoreErrorCodes::IdWrongFormat,
			"Id has a wrong format. Make sure to follow the uuid v4 or v7 format.",
		)
	})?;

	if let Some(v) = uuid.get_version() {
		match v {
			Version::Random => {
				//uuid v4
				return Ok(());
			},
			Version::SortRand => {
				//uuid v7
				return Ok(());
			},
			_ => {},
		}
	}

	Err(ServerCoreError::new_msg(
		400,
		CoreErrorCodes::IdWrongFormat,
		"Id has a wrong format. Make sure to follow the uuid v4 or v7 format.",
	))
}

pub fn create_id_v4() -> String
{
	Uuid::new_v4().to_string()
}

pub fn create_id() -> String
{
	Uuid::now_v7().to_string()
}
