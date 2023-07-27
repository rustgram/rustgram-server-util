use rustgram_server_util::static_var::file_handler;

#[tokio::main]
async fn main()
{
	dotenv::dotenv().ok();

	file_handler::init_storage().await;
}
