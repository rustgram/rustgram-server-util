use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::db::{get_in, StringEntity, TransactionData};
use rustgram_server_util::static_var::db;
use rustgram_server_util::static_var::db::db;
use rustgram_server_util::{get_time, set_params};

#[derive(Debug)]
#[cfg_attr(feature = "mysql", derive(rustgram_server_util_macros::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util_macros::Sqlite))]
pub struct TestData
{
	id: String,
	_name: String,
	_time: u128,
}

#[tokio::test]
async fn tests()
{
	println!("-----------");
	println!("init");
	init().await;

	println!("-----------");
	println!("db_insert_and_fetch");
	test_10_db_insert_and_fetch().await;

	println!("-----------");
	println!("insert_and_fetch_with_get_ins");
	test_12_insert_and_fetch_with_get_ins().await;

	println!("-----------");
	println!("bulk_insert");
	test_13_bulk_insert().await;

	println!("-----------");
	println!("tx_exec");
	test_14_tx_exec().await;

	println!("-----------");
	println!("clean up");
	clean_up().await;
	println!("-----------");
}

async fn init()
{
	dotenv::dotenv().ok();

	db::init_db().await;

	//language=SQL
	let sql = r"
CREATE table IF NOT EXISTS test (
    `id` varchar(36) NOT NULL,
    `name` text DEFAULT NULL,
    `time` text DEFAULT NULL
)";

	db().exec_non_param(sql).await.unwrap();

	#[cfg(feature = "mysql")]
	//language=SQL
	let sql = "SHOW TABLES LIKE 'test'";

	#[cfg(feature = "sqlite")]
	let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'test'";

	let res: Option<StringEntity> = db().query_first_non_param(sql).await.unwrap();

	assert!(res.is_some());
}

async fn test_10_db_insert_and_fetch()
{
	dotenv::dotenv().ok();

	//language=SQLx
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

	//test query first
	let test_datum: Option<TestData> = db()
		.query_first(sql, set_params!(id.clone()))
		.await
		.unwrap();

	assert_eq!(test_datum.unwrap().id, id);

	//test without result
	let test_datum: Option<TestData> = db()
		.query_first(sql, set_params!(id.clone() + "123"))
		.await
		.unwrap();

	let not_found_datum = test_datum.is_none();

	assert!(not_found_datum);
}

async fn test_12_insert_and_fetch_with_get_ins()
{
	dotenv::dotenv().ok();

	//two inserts
	//language=SQLx
	let sql = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

	let id1 = create_id();
	let name1 = "hello1".to_string();
	let time1 = get_time().unwrap();

	db().exec(sql, set_params!(id1.clone(), name1, time1.to_string()))
		.await
		.unwrap();

	//language=SQLx
	let sql = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

	let id2 = create_id();
	let name2 = "hello2".to_string();
	let time2 = get_time().unwrap();

	db().exec(sql, set_params!(id2.clone(), name2, time2.to_string()))
		.await
		.unwrap();

	let params = vec![id1.clone(), id2.clone()];

	let ins = get_in(&params);

	//language=SQLx
	let sql = format!("SELECT * FROM test WHERE id IN ({}) ORDER BY name", ins);

	let test_data: Vec<TestData> = db().query_string(sql, params).await.unwrap();

	println!("out get in: {:?}", test_data);

	assert_eq!(test_data.len(), 2);
	assert_eq!(test_data[0].id, id1);
	assert_eq!(test_data[1].id, id2);
}

async fn test_13_bulk_insert()
{
	dotenv::dotenv().ok();

	//do this extra because we need the ids later to check if this values are in the db
	let id1 = create_id();
	let id2 = create_id();
	let id3 = create_id();

	let t1 = TestData {
		id: id1.to_string(),
		_name: "hello".to_string(),
		_time: get_time().unwrap(),
	};

	let t2 = TestData {
		id: id2.to_string(),
		_name: "hello2".to_string(),
		_time: get_time().unwrap(),
	};

	let t3 = TestData {
		id: id3.to_string(),
		_name: "hello3".to_string(),
		_time: get_time().unwrap(),
	};

	db().bulk_insert(false, "test", &["id", "name", "time"], vec![t1, t2, t3], |ob| {
		set_params!(ob.id, ob._name, ob._time.to_string())
	})
	.await
	.unwrap();

	//check if the values are in the db
	let params = vec![id1.clone(), id2.clone(), id3.clone()];

	let ins = get_in(&params);

	//language=SQLx
	let sql = format!("SELECT * FROM test WHERE id IN ({}) ORDER BY name", ins);

	let test_data: Vec<TestData> = db().query_string(sql, params).await.unwrap();

	println!("out bulk insert: {:?}", test_data);

	assert_eq!(test_data.len(), 3);
	assert_eq!(test_data[0].id, id1);
	assert_eq!(test_data[1].id, id2);
	assert_eq!(test_data[2].id, id3);
}

async fn test_14_tx_exec()
{
	dotenv::dotenv().ok();

	//language=SQLx
	let sql = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

	let id1 = create_id();
	let name1 = "hello1".to_string();
	let time1 = get_time().unwrap();

	//language=SQLx
	let sql2 = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

	let id2 = create_id();
	let name2 = "hello2".to_string();
	let time2 = get_time().unwrap();

	//language=SQLx
	let sql3 = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

	let id3 = create_id();
	let name3 = "hello3".to_string();
	let time3 = get_time().unwrap();

	db().exec_transaction(vec![
		TransactionData {
			sql,
			params: set_params!(id1.clone(), name1, time1.to_string()),
		},
		TransactionData {
			sql: sql2,
			params: set_params!(id2.clone(), name2, time2.to_string()),
		},
		TransactionData {
			sql: sql3,
			params: set_params!(id3.clone(), name3, time3.to_string()),
		},
	])
	.await
	.unwrap();

	let params = vec![id1.clone(), id2.clone(), id3.clone()];

	let ins = get_in(&params);

	//language=SQLx
	let sql = format!("SELECT * FROM test WHERE id IN ({}) ORDER BY name", ins);

	let test_data: Vec<TestData> = db().query_string(sql, params).await.unwrap();

	println!("out get in: {:?}", test_data);

	assert_eq!(test_data.len(), 3);
	assert_eq!(test_data[0].id, id1);
	assert_eq!(test_data[1].id, id2);
	assert_eq!(test_data[2].id, id3);
}

async fn clean_up()
{
	dotenv::dotenv().ok();

	db::init_db().await;

	//language=SQLx
	let sql = "DROP TABLE test";

	db().exec_non_param(sql).await.unwrap();

	#[cfg(feature = "mysql")]
	//language=SQL
	let sql = "SHOW TABLES LIKE 'test'";

	#[cfg(feature = "sqlite")]
	let sql = "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'test'";

	let res: Option<StringEntity> = db().query_first_non_param(sql).await.unwrap();

	assert!(res.is_none());
}
