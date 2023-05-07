use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::cache::Cache;
use crate::get_time_in_sec;
use crate::res::AppRes;

pub struct CacheData<T: 'static + Clone>
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
				if v.ttl < get_time_in_sec().unwrap() as usize {
					return Ok(None);
				}

				Ok(Some(v.value.clone()))
			},
			None => Ok(None),
		}
	}

	async fn add(&self, key: String, value: T, ttl: usize) -> AppRes<()>
	{
		let ttl = ttl + get_time_in_sec().unwrap() as usize;

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
}

/**
Init th cache as async

Must be async for RwLock from tokio.
*/
pub async fn init_cache<T: 'static + Clone + Send + Sync>() -> Box<dyn Cache<T>>
{
	Box::new(ArrayCache::new())
}
