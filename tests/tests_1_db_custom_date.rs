use rustgram_server_util::db::custom_types::date_str::{DateStr, DateTimeMilliStr, DateTimeStr, TimeSinglePositionStr};
use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::db::StringEntity;
use rustgram_server_util::{db, set_params};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "mysql", derive(rustgram_server_util_macros::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util_macros::Sqlite))]
pub struct TestData
{
	id: String,
	date: DateStr,
	hour: TimeSinglePositionStr,
	min: TimeSinglePositionStr,
	sec: TimeSinglePositionStr,
	date_time: DateTimeStr,
	date_time_milli: DateTimeMilliStr,
}

#[tokio::test]
async fn tests()
{
	dotenv::dotenv().ok();

	println!("-----------");
	println!("init");
	init().await;

	println!("-----------");
	println!("db_insert_and_fetch");
	test_10_db_insert_and_fetch().await;

	println!("-----------");
	println!("clean up");
	clean_up().await;
	println!("-----------");
}

async fn init()
{
	db::init_db().await;

	#[cfg(feature = "mysql")]
	//language=SQL
	let sql = r"
CREATE table IF NOT EXISTS test_date (
    `id` varchar(36) NOT NULL,
    `date` DATE NOT NULL,
    `hour` INT NOT NULL,
    `min` INT NOT NULL,
    `sec` INT NOT NULL,
    `date_time` DATETIME NOT NULL,
    `date_time_milli` DATETIME(3) NOT NULL
)";

	#[cfg(feature = "sqlite")]
	//language=SQL
	let sql = r"
CREATE table IF NOT EXISTS test_date (
    `id` varchar(36) NOT NULL,
    `date` text DEFAULT NULL,
    `hour` INT NOT NULL,
    `min` INT NOT NULL,
    `sec` INT NOT NULL,
    `date_time` text NOT NULL,
    `date_time_milli` text NOT NULL
)";

	db::exec_non_param(sql).await.unwrap();

	#[cfg(feature = "mysql")]
	//language=SQL
	let sql = "SHOW TABLES LIKE 'test_date'";

	#[cfg(feature = "sqlite")]
	let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'test_date'";

	let res: Option<StringEntity> = db::query_first_non_param(sql).await.unwrap();

	assert!(res.is_some());
}

async fn test_10_db_insert_and_fetch()
{
	//two inserts
	//language=SQL
	let sql = "INSERT INTO test_date (id, date, hour, min, sec, date_time, date_time_milli) VALUES (?,?,?,?,?,?,?)";

	let id = create_id();
	let date = "2024-03-01";
	let hour = 5;
	let min = 6;
	let sec = 4;
	let date_time = "2024-03-01 06:07:05";
	let date_time_milli = "2024-03-01 06:07:05.062";

	db::exec(
		sql,
		set_params!(
			id.clone(),
			date.to_string(),
			hour,
			min,
			sec,
			date_time.to_string(),
			date_time_milli.to_string()
		),
	)
	.await
	.unwrap();

	//language=SQL
	let sql = "SELECT * FROM test_date WHERE id = ?";

	let test_datum: TestData = db::query_first(sql, set_params!(id))
		.await
		.unwrap()
		.unwrap();

	println!("{:?}", test_datum);

	assert_eq!(test_datum.date.to_string(), date.to_string());

	let str = serde_json::to_string(&test_datum).unwrap();

	println!("{str}");
}

async fn clean_up()
{
	db::init_db().await;

	//language=SQLx
	let sql = "DROP TABLE test_date";

	db::exec_non_param(sql).await.unwrap();

	#[cfg(feature = "mysql")]
	//language=SQL
	let sql = "SHOW TABLES LIKE 'test_date'";

	#[cfg(feature = "sqlite")]
	let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'test_date'";

	let res: Option<StringEntity> = db::query_first_non_param(sql).await.unwrap();

	assert!(res.is_none());
}
