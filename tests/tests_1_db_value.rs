use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::db::value::{DbRow, OutputRow, Value};
use rustgram_server_util::db::StringEntity;
use rustgram_server_util::error::{server_err, CoreErrorCodes};
use rustgram_server_util::res::AppRes;
use rustgram_server_util::{db, set_params_vec_outer};

#[tokio::test]
async fn tests()
{
	dotenv::dotenv().ok();

	println!("-----------");
	println!("init");
	init().await;

	println!("-----------");
	println!("insert and fetch from db");
	test_1_insert_and_fetch().await;

	println!("-----------");
	println!("insert form json input");
	test_2_from_json_input().await;

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
    first_name text NOT NULL
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
    first_name text NOT NULL
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

async fn test_1_insert_and_fetch()
{
	//language=SQL
	let sql = "INSERT INTO test_date (id, date, hour, min, sec, date_time, first_name) VALUES (?,?,?,?,?,?,?)";

	let id = create_id();

	let value_arr = vec![
		id.clone().into(),
		Value::str_to_date("2024-03-01").unwrap(),
		5.into(),
		6.into(),
		4.into(),
		Value::str_to_date_time("2024-03-01 06:07:05").unwrap(),
		"Bla".into(),
	];

	db::exec(sql, set_params_vec_outer!(value_arr))
		.await
		.unwrap();

	//language=SQL
	let sql = "SELECT * FROM test_date WHERE id = ?";

	let select_value_arr: Vec<Value> = vec![id.into()];

	let out: OutputRow = db::query_first(sql, set_params_vec_outer!(select_value_arr))
		.await
		.unwrap()
		.unwrap();

	let row: DbRow = out.into();

	println!("{:?}", row);
}

fn input_handle_object(v: Value) -> AppRes<Vec<Value>>
{
	//TODO search for the right order because the deserializer ordered it in alphabetic order
	let input_arr = if let Value::Object(o) = v {
		let mut arr = Vec::with_capacity(o.len() + 1); //+1 for the id

		for (_key, value) in o {
			if let Value::Array(_array) = value {
				//TODO handle array by checking the key if it must be an array
			} else if let Value::Object(_object) = value {
				//TODO handle object by checking the key if it must be an object
			} else {
				arr.push(value);
			}
		}

		arr
	} else {
		return Err(server_err(400, CoreErrorCodes::JsonParse, "Must be an object"));
	};

	//TODO return a struct with all values for the different db inserts or updates. for each as vec of values
	Ok(input_arr)
}

async fn test_2_from_json_input()
{
	let input = r#"{
		"date": "2024-03-01",
		"date_time": "2024-03-01 06:07:05",
		"first_name": "Hello",
		"hour": 1,
		"min": 3,
		"sec": 30
	}"#;

	let v = Value::from_json(input).unwrap();

	println!("{:?}", v);

	//transform to db input
	let mut input_arr = input_handle_object(v).unwrap();

	//language=SQL
	let sql = "INSERT INTO test_date (date, date_time, first_name, hour, min, sec, id) VALUES (?,?,?,?,?,?,?)";

	let id = create_id();

	input_arr.push(id.clone().into());

	db::exec(sql, set_params_vec_outer!(input_arr))
		.await
		.unwrap();

	//language=SQL
	let sql = "SELECT * FROM test_date WHERE id = ?";

	let select_value_arr: Vec<Value> = vec![id.into()];

	let out: OutputRow = db::query_first(sql, set_params_vec_outer!(select_value_arr))
		.await
		.unwrap()
		.unwrap();

	let row: DbRow = out.into();

	println!("{:?}", row);
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
