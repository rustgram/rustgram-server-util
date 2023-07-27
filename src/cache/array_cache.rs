use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::cache::Cache;
use crate::get_time_in_sec;
use crate::res::AppRes;

struct CacheData<T: 'static + Clone>
{
	value: T,
	ttl: usize,
}

/**
# Simple Array Cache with Multithreaded support

https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton
with RwLock instead of Mutex
 */
pub struct ArrayCache<T: 'static + Clone>
{
	//https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html
	cache: RwLock<HashMap<String, CacheData<T>>>,
}

impl<T: 'static + Clone> ArrayCache<T>
{
	pub fn new() -> Self
	{
		Self::default()
	}
}

impl<T: 'static + Clone> Default for ArrayCache<T>
{
	fn default() -> Self
	{
		Self {
			cache: RwLock::new(HashMap::<String, CacheData<T>>::new()),
		}
	}
}

#[async_trait]
impl<T: 'static + Clone + Send + Sync> Cache<T> for ArrayCache<T>
{
	async fn get(&self, key: &str) -> AppRes<Option<T>>
	{
		let cache = self.cache.read().await;

		match cache.get(key) {
			Some(v) => {
				if v.ttl < get_time_in_sec()? as usize {
					return Ok(None);
				}

				Ok(Some(v.value.clone()))
			},
			None => Ok(None),
		}
	}

	async fn add(&self, key: String, value: T, ttl: usize) -> AppRes<()>
	{
		let ttl = ttl + get_time_in_sec()? as usize;

		self.cache.write().await.insert(
			key,
			CacheData {
				ttl,
				value,
			},
		);

		Ok(())
	}

	async fn delete(&self, key: &str) -> AppRes<()>
	{
		self.cache.write().await.remove(key);

		Ok(())
	}

	async fn delete_multiple(&self, keys: &[&str]) -> AppRes<()>
	{
		let mut c = self.cache.write().await;

		for key in keys {
			c.remove(*key);
		}

		Ok(())
	}
}
