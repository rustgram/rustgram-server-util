use std::env;
use std::future::Future;
use std::time::Duration;

use redis::{FromRedisValue, ToRedisArgs};
use rustgram_server_util::cache::{ArrayCache, Cache, RedisCache};
use rustgram_server_util::res::AppRes;
use tokio::sync::OnceCell;

const KEY: &str = "test_key";
const VALUE: &str = "test_value";

static CACHE: OnceCell<Box<dyn Cache<String>>> = OnceCell::const_new();

/**
Init th cache as async

Must be async for RwLock from tokio.
 */
async fn array_cache_init_cache<T: 'static + Clone + Send + Sync>() -> Box<dyn Cache<T>>
{
	#[cfg(debug_assertions)]
	println!("init array cache");

	Box::new(ArrayCache::new())
}

async fn redis_init_cache<T: 'static + Clone + Send + Sync + FromRedisValue + ToRedisArgs>() -> Box<dyn Cache<T>>
{
	let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

	#[cfg(debug_assertions)]
	println!("init redis");

	Box::new(RedisCache::new(&redis_url))
}

async fn init_cache_with_env()
{
	let cache = env::var("CACHE").unwrap_or_else(|_| "1".to_string());

	match cache.as_str() {
		"1" => {
			CACHE.get_or_init(array_cache_init_cache::<String>).await;
		},
		"2" => {
			CACHE.get_or_init(redis_init_cache::<String>).await;
		},
		_ => panic!("Cache init error: Please choose either `1` for array cache or `2` for redis cache."),
	}
}

#[allow(clippy::needless_lifetimes)]
pub fn get<'a>(key: &'a str) -> impl Future<Output = AppRes<Option<String>>> + 'a
{
	let cache = CACHE.get().unwrap();

	cache.get(key)
}

pub fn add(key: String, value: String, ttl: usize) -> impl Future<Output = AppRes<()>>
{
	let cache = CACHE.get().unwrap();

	cache.add(key, value, ttl)
}

#[allow(clippy::needless_lifetimes)]
pub fn delete<'a>(key: &'a str) -> impl Future<Output = AppRes<()>> + 'a
{
	let cache = CACHE.get().unwrap();

	cache.delete(key)
}

pub fn delete_multiple<'a>(keys: &'a [&str]) -> impl Future<Output = AppRes<()>> + 'a
{
	let cache = CACHE.get().unwrap();

	cache.delete_multiple(keys)
}

#[tokio::test]
async fn tests()
{
	dotenv::dotenv().ok();

	println!("-----------");
	println!("init");
	init_cache_with_env().await;

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
	add(KEY.into(), VALUE.into(), 200).await.unwrap();
}

async fn get_value()
{
	let value = get(KEY).await.unwrap();

	assert_eq!(value, Some(VALUE.to_string()));
}

async fn store_value_with_ttl()
{
	//use a very short ttl
	add(KEY.to_string() + "2", VALUE.into(), 1).await.unwrap();

	//the value must no be valid after 1 sec
	tokio::time::sleep(Duration::from_secs(2)).await;

	let value = get(&(KEY.to_string() + "2")).await.unwrap();
	assert_eq!(value, None);
}

async fn delete_value()
{
	//value should be there
	let value = get(KEY).await.unwrap();
	assert_eq!(value, Some(VALUE.to_string()));

	delete(KEY).await.unwrap();

	let value = get(KEY).await.unwrap();
	assert_eq!(value, None);
}

async fn delete_multiple_test()
{
	//create multiple keys
	let key_arr = ["test_key_1", "test_key_2", "test_key_3", "test_key_4", "test_key_5"];

	for k in key_arr {
		add(k.into(), VALUE.into(), 200).await.unwrap();
	}

	//values mut be there
	for k in key_arr {
		let value = get(k).await.unwrap();
		assert_eq!(value, Some(VALUE.to_string()));
	}

	delete_multiple(&key_arr).await.unwrap();

	for k in key_arr {
		let value = get(k).await.unwrap();
		assert_eq!(value, None);
	}
}
