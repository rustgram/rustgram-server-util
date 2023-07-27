use std::env;
use std::future::Future;

use redis::{FromRedisValue, ToRedisArgs};
use rustgram_server_util::cache::{ArrayCache, Cache, RedisCache};
use rustgram_server_util::res::AppRes;
use tokio::sync::OnceCell;

static CACHE: OnceCell<Box<dyn Cache<String>>> = OnceCell::const_new();

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

async fn init_cache()
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

#[tokio::main]
async fn main()
{
	dotenv::dotenv().ok();

	init_cache().await;

	add("key".to_string(), "value".to_string(), 200)
		.await
		.unwrap();

	let value = get("key").await.unwrap();

	assert_eq!(value, Some("value".to_string()));

	delete("key").await.unwrap();

	let value = get("key").await.unwrap();
	assert_eq!(value, None);
}
