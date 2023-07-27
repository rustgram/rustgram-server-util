mod local_storage;

use async_trait::async_trait;
pub use local_storage::LocalStorage;
use rustgram::{Request, Response};

use crate::error::ServerCoreError;

#[async_trait]
pub trait FileHandler: Send + Sync
{
	async fn get_part(&self, part_id: &str, content_type: Option<&str>) -> Result<Response, ServerCoreError>;

	async fn upload_part(&self, req: Request, part_id: &str, max_chunk_size: usize) -> Result<usize, ServerCoreError>;

	async fn delete_part(&self, part_id: &str) -> Result<(), ServerCoreError>;

	async fn delete_parts(&self, parts: &[String]) -> Result<(), ServerCoreError>;
}
