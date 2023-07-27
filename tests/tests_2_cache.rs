use std::time::Duration;

use rustgram_server_util::cache;

const KEY: &str = "test_key";
const VALUE: &str = "test_value";

#[tokio::test]
async fn tests()
{
	dotenv::dotenv().ok();

	println!("-----------");
	println!("init");
	cache::init_cache().await;

	println!("-----------");
	println!("store value");
	store_value().await;

	println!("-----------");
	println!("get value");
	get_value().await;

	println!("-----------");
	println!("store ttl value");
	store_value_with_ttl().await;

	println!("-----------");
	println!("delete value");
	delete_value().await;

	println!("-----------");
	println!("delete multiple value");
	delete_multiple_test().await;
}

async fn store_value()
{
	cache::add(KEY.into(), VALUE.into(), 200).await.unwrap();
}

async fn get_value()
{
	let value = cache::get(KEY).await.unwrap();

	assert_eq!(value, Some(VALUE.to_string()));
}

async fn store_value_with_ttl()
{
	//use a very short ttl
	cache::add(KEY.to_string() + "2", VALUE.into(), 1)
		.await
		.unwrap();

	//the value must no be valid after 1 sec
	tokio::time::sleep(Duration::from_secs(2)).await;

	let value = cache::get(&(KEY.to_string() + "2")).await.unwrap();
	assert_eq!(value, None);
}

async fn delete_value()
{
	//value should be there
	let value = cache::get(KEY).await.unwrap();
	assert_eq!(value, Some(VALUE.to_string()));

	cache::delete(KEY).await.unwrap();

	let value = cache::get(KEY).await.unwrap();
	assert_eq!(value, None);
}

async fn delete_multiple_test()
{
	//create multiple keys
	let key_arr = ["test_key_1", "test_key_2", "test_key_3", "test_key_4", "test_key_5"];

	for k in key_arr {
		cache::add(k.into(), VALUE.into(), 200).await.unwrap();
	}

	//values mut be there
	for k in key_arr {
		let value = cache::get(k).await.unwrap();
		assert_eq!(value, Some(VALUE.to_string()));
	}

	cache::delete_multiple(&key_arr).await.unwrap();

	for k in key_arr {
		let value = cache::get(k).await.unwrap();
		assert_eq!(value, None);
	}
}
