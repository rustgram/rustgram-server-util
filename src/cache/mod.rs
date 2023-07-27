use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::res::AppRes;

mod array_cache;
mod redis_cache;

pub use array_cache::ArrayCache;
pub use redis_cache::RedisCache;

#[cfg(feature = "static_var")]
pub use crate::static_var::cache::*;

#[async_trait]
pub trait Cache<T: 'static + Clone>: Send + Sync
{
	async fn get(&self, key: &str) -> AppRes<Option<T>>;

	async fn add(&self, key: String, value: T, ttl: usize) -> AppRes<()>;

	async fn delete(&self, key: &str) -> AppRes<()>;

	async fn delete_multiple(&self, keys: &[&str]) -> AppRes<()>;
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
