use rustgram_server_util::file::init_storage;

#[tokio::main]
async fn main()
{
	dotenv::dotenv().ok();

	init_storage().await;
}
