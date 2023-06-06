use std::env;
use std::future::Future;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::res::AppRes;

mod array_cache;

static CACHE: OnceCell<Box<dyn Cache<String>>> = OnceCell::const_new();

#[async_trait]
pub trait Cache<T: 'static + Clone>: Send + Sync
{
	async fn get(&self, key: &str) -> AppRes<Option<T>>;

	async fn add(&self, key: String, value: T, ttl: usize) -> AppRes<()>;

	async fn delete(&self, key: &str) -> AppRes<()>;

	async fn delete_multiple(&self, keys: &[&str]) -> AppRes<()>;
}

pub async fn init_cache()
{
	let cache = env::var("CACHE").unwrap();

	if cache.as_str() == "1" {
		CACHE.get_or_init(array_cache::init_cache::<String>).await;
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

#[derive(Serialize, Deserialize)]
pub enum CacheVariant<T>
{
	Some(T),
	None,
}

pub const DEFAULT_TTL: usize = 60 * 60; //1h (60 sec * 60 min)
pub const LONG_TTL: usize = 60 * 60 * 24; //24 h
pub const SHORT_TTL: usize = 60 * 5; //5 min
