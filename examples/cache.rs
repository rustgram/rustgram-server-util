use rustgram_server_util::static_var::cache;

#[tokio::main]
async fn main()
{
	dotenv::dotenv().ok();

	cache::init_cache().await;

	cache::add("key".to_string(), "value".to_string(), 200)
		.await
		.unwrap();

	let value = cache::get("key").await.unwrap();

	assert_eq!(value, Some("value".to_string()));

	cache::delete("key").await.unwrap();

	let value = cache::get("key").await.unwrap();
	assert_eq!(value, None);
}
