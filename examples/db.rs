use rustgram_server_util::db::id_handling::create_id;
use rustgram_server_util::{db, get_time, set_params};

/*
You can use this derive:
#[derive(rustgram_server_util::DB)]

instead of
#[cfg_attr(feature = "mysql", derive(rustgram_server_util_macros::MariaDb))]
#[cfg_attr(feature = "sqlite", derive(rustgram_server_util_macros::Sqlite))]

when using the derive_macro feature
 */

#[derive(Debug, rustgram_server_util::DB)]
pub struct TestData
{
	id: String,
	_name: String,
	_time: u128,
}

#[tokio::main]
async fn main()
{
	dotenv::dotenv().ok();

	db::init_db().await;

	let sql = "INSERT INTO test (id, name, time) VALUES (?,?,?)";

	let id = create_id();
	let name = "hello".to_string();
	let time = get_time().unwrap();

	db::exec(sql, set_params!(id.clone(), name, time.to_string()))
		.await
		.unwrap();

	//fetch the new test data
	//language=SQLx
	let sql = "SELECT * FROM test WHERE id = ?";

	let test_data: Vec<TestData> = db::query(sql, set_params!(id.clone())).await.unwrap();

	println!("out: {:?}", test_data);

	assert_eq!(test_data.len(), 1);
	assert_eq!(test_data[0].id, id);
}
