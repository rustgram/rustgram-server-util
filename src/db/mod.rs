use std::error::Error;

use crate::error::{CoreErrorCodes, ServerCoreError, ServerErrorConstructor};

pub mod id_handling;
#[cfg(feature = "mysql")]
mod mariadb;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "mysql")]
pub use mysql_async as mysql_async_export;
#[cfg(feature = "mysql")]
pub use mysql_common as mysql_common_export;
#[cfg(feature = "sqlite")]
pub use rusqlite as rusqlite_export;

#[cfg(feature = "mysql")]
pub use self::mariadb::{Mariadb as Db, TransactionData};
#[cfg(feature = "sqlite")]
pub use self::sqlite::{FormSqliteRowError, FromSqliteRow, Sqlite as Db, TransactionData};
#[cfg(feature = "static_var")]
pub use crate::static_var::db::*;

#[cfg(feature = "mysql")]
pub type Params = mysql_common::params::Params;

#[cfg(feature = "sqlite")]
pub type Params = Vec<rusqlite::types::Value>;

#[allow(clippy::useless_format)]
/**
# Returns a ? string for multiple parameter

````rust_sample
	let ids = vec!["lol", "abc", "123"];

	let ins = get_in(&ids);

	println!("{:?}", ins);

	//prints "?,?,?"
````
 */
pub fn get_in<T>(objects: &[T]) -> String
{
	format!(
		"{}",
		objects
			.iter()
			.map(|_| "?".to_string())
			.collect::<Vec<_>>()
			.join(",")
	)
}

fn db_query_err<E: Error>(e: &E) -> ServerCoreError
{
	ServerCoreError::new_msg_and_debug(
		422,
		CoreErrorCodes::DbQuery,
		"db error",
		Some(format!("db fetch err, {:?}", e)),
	)
}

fn db_exec_err<E: Error>(e: &E) -> ServerCoreError
{
	ServerCoreError::new_msg_and_debug(
		422,
		CoreErrorCodes::DbExecute,
		"db error",
		Some(format!("db execute err, {:?}", e)),
	)
}

fn db_bulk_insert_err<E: Error>(e: &E) -> ServerCoreError
{
	ServerCoreError::new_msg_and_debug(
		422,
		CoreErrorCodes::DbBulkInsert,
		"db error",
		Some(format!("db bulk insert err, {:?}", e)),
	)
}

fn db_tx_err<E: Error>(e: &E) -> ServerCoreError
{
	ServerCoreError::new_msg_and_debug(
		422,
		CoreErrorCodes::DbTx,
		"Db error",
		Some(format!("Db transaction error: {:?}", e)),
	)
}

/**
# Tuple for async-mysql params

transform the values like into_params_impl from mysql_common::params

 */
#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! set_params {
	($( $param:expr ),+ $(,)?) => {{
		 $crate::db::mysql_common_export::params::Params::Positional(vec![
			 $($param.into(),)*
         ])
	}};
}

#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! set_params_vec {
	($vec:expr) => {{
		let mut out: Vec<$crate::db::mysql_common_export::value::Value> = Vec::with_capacity($vec.len());

		for inp in $vec {
			out.push(inp.0.into());
		}

		$crate::db::mysql_common_export::params::Params::Positional(out)
	}};
}

#[cfg(feature = "mysql")]
#[macro_export]
macro_rules! set_params_vec_outer {
	($vec:expr) => {{
		let mut out: Vec<$crate::db::mysql_common_export::value::Value> = Vec::with_capacity($vec.len());

		for inp in $vec {
			out.push(inp.into());
		}

		$crate::db::mysql_common_export::params::Params::Positional(out)
	}};
}

/**
# The sql params for sqlite

 */
#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! set_params {
	($( $param:expr ),+ $(,)?) => {
		vec![
			$($crate::db::rusqlite_export::types::Value::from($param),)*
		]
	};
}

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! set_params_vec {
	($vec:expr) => {{
		let mut tmp = Vec::with_capacity($vec.len());

		for inp in $vec {
			tmp.push($crate::db::rusqlite_export::types::Value::from(inp.0))
		}

		tmp
	}};
}

#[cfg(feature = "sqlite")]
#[macro_export]
macro_rules! set_params_vec_outer {
	($vec:expr) => {{
		let mut tmp = Vec::with_capacity($vec.len());

		for inp in $vec {
			tmp.push($crate::db::rusqlite_export::types::Value::from(inp))
		}

		tmp
	}};
}

//__________________________________________________________________________________________________

//impl for one tuple structs

pub struct TupleEntity<T>(pub T);

#[cfg(feature = "mysql")]
impl<T: mysql_async::prelude::FromValue> mysql_async::prelude::FromRow for TupleEntity<T>
{
	fn from_row_opt(mut row: mysql_async::Row) -> Result<Self, mysql_async::FromRowError>
	where
		Self: Sized,
	{
		Ok(Self(match row.take_opt(0) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err(mysql_async::FromValueError(_value)) => return Err(mysql_async::FromRowError(row)),
				}
			},
			None => return Err(mysql_async::FromRowError(row)),
		}))
	}
}

#[cfg(feature = "sqlite")]
impl<T: rusqlite::types::FromSql> crate::db::FromSqliteRow for TupleEntity<T>
{
	fn from_row_opt(row: &rusqlite::Row) -> Result<Self, crate::db::FormSqliteRowError>
	where
		Self: Sized,
	{
		Ok(Self(match row.get(0) {
			Ok(v) => v,
			Err(e) => {
				return Err(crate::db::FormSqliteRowError {
					msg: format!("{:?}", e),
				})
			},
		}))
	}
}

pub type StringEntity = TupleEntity<String>;

pub type I32Entity = TupleEntity<i32>;

pub type I64Entity = TupleEntity<i64>;
