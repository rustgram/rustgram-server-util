#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "sqlite")]
mod sqlite;

use std::env;

use tokio::sync::OnceCell;

#[cfg(feature = "mysql")]
pub use self::mysql::{
	bulk_insert,
	exec,
	exec_non_param,
	exec_string,
	exec_string_non_param,
	exec_transaction,
	query,
	query_first,
	query_first_non_param,
	query_first_string,
	query_first_string_non_param,
	query_non_param,
	query_string,
	query_string_non_param,
};
#[cfg(feature = "sqlite")]
pub use self::sqlite::{
	bulk_insert,
	exec,
	exec_non_param,
	exec_string,
	exec_string_non_param,
	exec_transaction,
	query,
	query_first,
	query_first_non_param,
	query_first_string,
	query_first_string_non_param,
	query_non_param,
	query_string,
	query_string_non_param,
};
use crate::db::Db;

static DB_CONN: OnceCell<Db> = OnceCell::const_new();

#[cfg(feature = "mysql")]
async fn init_mariadb() -> Db
{
	let user = env::var("DB_USER").unwrap();
	let pw = env::var("DB_PASS").unwrap();
	let mysql_host = env::var("DB_HOST").unwrap();
	let db_name = env::var("DB_NAME").unwrap();
	let db_port = env::var("DB_PORT").ok(); //option

	#[cfg(feature = "mysql")]
	Db::new(
		&user,
		&pw,
		&mysql_host,
		&db_name,
		db_port.map(|o| if o.is_empty() { 3306 } else { o.parse().unwrap() }),
	)
}

#[cfg(feature = "sqlite")]
async fn init_sqlite() -> Db
{
	#[cfg(feature = "sqlite")]
	Db::new(&env::var("DB_PATH").unwrap())
}

pub async fn init_db()
{
	#[cfg(feature = "sqlite")]
	DB_CONN.get_or_init(init_sqlite).await;

	#[cfg(feature = "mysql")]
	DB_CONN.get_or_init(init_mariadb).await;
}

pub fn db<'a>() -> &'a Db
{
	DB_CONN.get().unwrap()
}
