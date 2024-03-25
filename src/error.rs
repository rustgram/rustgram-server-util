use std::error::Error;
use std::fmt::{Display, Formatter};

pub trait ServerErrorCodes
{
	fn get_int_code(&self) -> u32;
}

#[derive(Debug)]
pub enum CoreErrorCodes
{
	IdWrongFormat,

	JsonToString,
	JsonParse,

	InputTooBig,

	UnexpectedTime,

	DbQuery,
	DbExecute,
	DbBulkInsert,
	DbTx,
	NoDbConnection,

	EmailMessage,
	EmailSend,

	NoParameter,
	NoUrlQuery,

	FileLocalOpen,
	FileRemove,
	FileSave,
	FileTooLarge,
	FileDownload,

	RedisError,

	DateStrParse,
}

impl ServerErrorCodes for CoreErrorCodes
{
	fn get_int_code(&self) -> u32
	{
		match self {
			CoreErrorCodes::IdWrongFormat => 1,

			CoreErrorCodes::JsonToString => 10,
			CoreErrorCodes::JsonParse => 11,
			CoreErrorCodes::InputTooBig => 12,
			CoreErrorCodes::UnexpectedTime => 13,

			CoreErrorCodes::NoDbConnection => 20,
			CoreErrorCodes::DbQuery => 21,
			CoreErrorCodes::DbExecute => 22,
			CoreErrorCodes::DbBulkInsert => 23,
			CoreErrorCodes::DbTx => 24,

			CoreErrorCodes::NoParameter => 40,
			CoreErrorCodes::NoUrlQuery => 41,

			CoreErrorCodes::EmailSend => 50,
			CoreErrorCodes::EmailMessage => 51,

			CoreErrorCodes::RedisError => 52,

			CoreErrorCodes::DateStrParse => 60,

			CoreErrorCodes::FileLocalOpen => 500,
			CoreErrorCodes::FileRemove => 501,
			CoreErrorCodes::FileSave => 502,
			CoreErrorCodes::FileDownload => 503,
			CoreErrorCodes::FileTooLarge => 504,
		}
	}
}

#[derive(Debug)]
pub struct ServerCoreError
{
	pub http_status_code: u16,
	pub error_code: u32,
	pub msg: &'static str,
	pub msg_owned: Option<String>, //msg will be ignored if this is set
	pub debug_msg: Option<String>,
}

impl Display for ServerCoreError
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
		match &self.msg_owned {
			Some(m) => write!(f, "Core error. Code: {}, Message: {}", self.error_code, m),
			None => write!(f, "Core error. Code: {}, Message: {}", self.error_code, self.msg),
		}
	}
}

impl Error for ServerCoreError {}

pub trait ServerErrorConstructor: Sized
{
	fn new(http_status_code: u16, error_code: impl ServerErrorCodes, msg: &'static str, msg_owned: Option<String>, debug_msg: Option<String>)
		-> Self;

	fn new_msg(http_status_code: u16, error_code: impl ServerErrorCodes, msg: &'static str) -> Self
	{
		ServerErrorConstructor::new(http_status_code, error_code, msg, None, None)
	}

	fn new_msg_owned(http_status_code: u16, error_code: impl ServerErrorCodes, msg_owned: String, debug_msg: Option<String>) -> Self
	{
		ServerErrorConstructor::new(http_status_code, error_code, "", Some(msg_owned), debug_msg)
	}

	fn new_msg_and_debug(http_status_code: u16, error_code: impl ServerErrorCodes, msg: &'static str, debug_msg: Option<String>) -> Self
	{
		ServerErrorConstructor::new(http_status_code, error_code, msg, None, debug_msg)
	}
}

impl ServerErrorConstructor for ServerCoreError
{
	fn new(http_status_code: u16, error_code: impl ServerErrorCodes, msg: &'static str, msg_owned: Option<String>, debug_msg: Option<String>)
		-> Self
	{
		Self {
			http_status_code,
			error_code: error_code.get_int_code(),
			msg,
			msg_owned,
			debug_msg,
		}
	}
}
