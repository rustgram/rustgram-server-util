use std::env;

use tokio::sync::OnceCell;

use crate::db::Db;

static DB_CONN: OnceCell<Db> = OnceCell::const_new();

#[cfg(feature = "mysql")]
async fn init_mariadb() -> Db
{
	let user = env::var("DB_USER").unwrap();
	let pw = env::var("DB_PASS").unwrap();
	let mysql_host = env::var("DB_HOST").unwrap();
	let db_name = env::var("DB_NAME").unwrap();

	#[cfg(feature = "mysql")]
	Db::new(&user, &pw, &mysql_host, &db_name)
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
