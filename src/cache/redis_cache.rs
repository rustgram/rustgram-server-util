use std::marker::PhantomData;

use async_trait::async_trait;
use redis::aio::Connection;
use redis::{AsyncCommands, Client, FromRedisValue, RedisError, ToRedisArgs};

use crate::cache::Cache;
use crate::error::{CoreErrorCodes, ServerCoreError, ServerErrorConstructor};
use crate::res::AppRes;

fn wrap_redis_error(e: RedisError) -> ServerCoreError
{
	ServerCoreError::new_msg_and_debug(
		400,
		CoreErrorCodes::RedisError,
		"Error with redis cache",
		Some(format!("Error with redis cache: {:?}", e)),
	)
}

pub struct RedisCache<T: 'static + Clone>
{
	p: PhantomData<T>,
	client: Client,
}

impl<T: 'static + Clone> RedisCache<T>
{
	pub fn new(redis_url: &str) -> Self
	{
		let client = Client::open(redis_url).unwrap();

		Self {
			p: Default::default(),
			client,
		}
	}

	pub async fn get_con(&self) -> Result<Connection, ServerCoreError>
	{
		self.client
			.get_async_connection()
			.await
			.map_err(wrap_redis_error)
	}
}

#[async_trait]
impl<T: 'static + Clone + Send + Sync + FromRedisValue + ToRedisArgs> Cache<T> for RedisCache<T>
{
	async fn get(&self, key: &str) -> AppRes<Option<T>>
	{
		let mut con = self.get_con().await?;

		con.get(key).await.map_err(wrap_redis_error)
	}

	async fn add(&self, key: String, value: T, ttl: usize) -> AppRes<()>
	{
		let mut con = self.get_con().await?;

		redis::pipe()
			.atomic()
			.set(&key, value)
			.expire(key, ttl)
			.query_async(&mut con)
			.await
			.map_err(wrap_redis_error)?;

		Ok(())
	}

	async fn delete(&self, key: &str) -> AppRes<()>
	{
		let mut con = self.get_con().await?;

		con.del(key).await.map_err(wrap_redis_error)?;

		Ok(())
	}

	async fn delete_multiple(&self, keys: &[&str]) -> AppRes<()>
	{
		let mut con = self.get_con().await?;

		con.del(keys).await.map_err(wrap_redis_error)?;

		Ok(())
	}
}
