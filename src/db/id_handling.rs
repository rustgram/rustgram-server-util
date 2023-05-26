use uuid::{Uuid, Version};

use crate::error::{CoreErrorCodes, ServerCoreError, ServerErrorConstructor};
use crate::res::AppRes;

pub fn check_id_format(id: &str) -> AppRes<()>
{
	let uuid = Uuid::try_parse(id).map_err(|_e| {
		ServerCoreError::new_msg(
			400,
			CoreErrorCodes::IdWrongFormat,
			"Id has a wrong format. Make sure to follow the uuid v4 format.",
		)
	})?;

	//uuid v4
	if let Some(Version::Random) = uuid.get_version() {
		return Ok(());
	}

	Err(ServerCoreError::new_msg(
		400,
		CoreErrorCodes::IdWrongFormat,
		"Id has a wrong format. Make sure to follow the uuid v4 format.",
	))
}

pub fn create_id() -> String
{
	Uuid::new_v4().to_string()
}
