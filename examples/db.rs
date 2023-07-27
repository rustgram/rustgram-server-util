use std::env;

use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::db::Db;
use rustgram_server_util::{get_time, set_params};
use tokio::sync::OnceCell;

static DB_CONN: OnceCell<Db> = OnceCell::const_new();

#[derive(Debug)]
#[cfg_attr(feature = "mysql", derive(rustgram_server_util::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util::Sqlite))]
pub struct TestData
{
	id: String,
	_name: String,
	_time: u128,
}

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

async fn init_db()
{
	#[cfg(feature = "sqlite")]
	DB_CONN.get_or_init(init_sqlite).await;

	#[cfg(feature = "mysql")]
	DB_CONN.get_or_init(init_mariadb).await;
}

fn db<'a>() -> &'a Db
{
	DB_CONN.get().unwrap()
}

#[tokio::main]
async fn main()
{
	dotenv::dotenv().ok();

	init_db().await;

	let sql = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

	let id = create_id();
	let name = "hello".to_string();
	let time = get_time().unwrap();

	db().exec(sql, set_params!(id.clone(), name, time.to_string()))
		.await
		.unwrap();

	//fetch the new test data
	//language=SQLx
	let sql = "SELECT * FROM test WHERE id = ?";

	let test_data: Vec<TestData> = db().query(sql, set_params!(id.clone())).await.unwrap();

	println!("out: {:?}", test_data);

	assert_eq!(test_data.len(), 1);
	assert_eq!(test_data[0].id, id);
}
