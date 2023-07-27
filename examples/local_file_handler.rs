use std::env;

use rustgram::{Request, Response};
use rustgram_server_util::error::ServerCoreError;
use rustgram_server_util::file::{FileHandler, LocalStorage};
use tokio::sync::OnceCell;

static FILE_HANDLER: OnceCell<Box<dyn FileHandler>> = OnceCell::const_new();

pub async fn init_storage()
{
	let storage = env::var("BACKEND_STORAGE").unwrap_or_else(|_| "0".to_string());

	if storage.as_str() == "0" {
		FILE_HANDLER.get_or_init(init_local_storage).await;
	}
}

async fn init_local_storage() -> Box<dyn FileHandler>
{
	let path = env::var("LOCAL_STORAGE_PATH").unwrap();

	Box::new(LocalStorage::new(path))
}

pub async fn get_part(part_id: &str) -> Result<Response, ServerCoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.get_part(part_id, None).await
}

pub async fn upload_part(req: Request, part_id: &str, max_chunk_size: usize) -> Result<usize, ServerCoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.upload_part(req, part_id, max_chunk_size).await
}

pub async fn delete_part(part_id: &str) -> Result<(), ServerCoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.delete_part(part_id).await
}

pub async fn delete_parts(parts: &[String]) -> Result<(), ServerCoreError>
{
	let handler = FILE_HANDLER.get().unwrap();

	handler.delete_parts(parts).await
}

#[tokio::main]
async fn main()
{
	dotenv::dotenv().ok();

	init_local_storage().await;
}
