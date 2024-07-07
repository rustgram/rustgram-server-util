use std::ffi::OsStr;
use std::path::Path;

use hyper::header::{HeaderValue, ACCEPT_ENCODING};
use rustgram::service::IntoResponse;
use rustgram::{Request, Response};
use tokio::sync::OnceCell;

use crate::error::{CoreErrorCodes, ServerCoreError, ServerErrorConstructor};
use crate::file::{FileHandler, LocalStorage};

pub static LOCAL_FILE_HANDLER: OnceCell<LocalStorage> = OnceCell::const_new();

pub async fn read_file(req: Request, path: &str) -> Response
{
	let file = if path.is_empty() || path == "/" { "index.html" } else { path };
	let mut file = file.to_owned();

	let ext = match Path::new(&file).extension() {
		Some(e) => {
			match OsStr::to_str(e) {
				Some(s) => s,
				None => return ServerCoreError::new_msg(404, CoreErrorCodes::PageNotFound, "Page not found").into_response(),
			}
		},
		None => {
			file += "/index.html";

			"html"
		},
	};

	let content_type = match ext {
		"html" => "text/html",
		"js" => "application/javascript",
		"wasm" => "application/wasm",
		"ico" => "image/x-icon",
		"png" => "image/png",
		"jpg" => "image/jpeg",
		"jpeg" => "image/jpeg",
		"svg" => "image/svg+xml",
		"woff2" => "font/woff2",
		"mp4" => "video/mp4",
		_ => return ServerCoreError::new_msg(404, CoreErrorCodes::PageNotFound, "Page not found").into_response(),
	};

	let headers = req.headers().get(ACCEPT_ENCODING);

	let encoding = match (ext == "js" || ext == "wasm", headers) {
		(true, Some(h)) => {
			if let Ok(h) = std::str::from_utf8(h.as_bytes()) {
				if h.contains("br") {
					//use brotli
					file += ".br";
					Some("br")
				} else if h.contains("gzip") {
					//use the zip js
					file += ".gz";
					Some("gzip")
				} else {
					None
				}
			} else {
				None
			}
		},
		_ => None,
	};

	let handler = LOCAL_FILE_HANDLER.get().unwrap();

	match handler.get_part(&file, Some(content_type)).await {
		Ok(mut res) => {
			let res_headers = res.headers_mut();

			//res_headers.insert("Cache-Control", HeaderValue::from_static("public, max-age=86400"));
			if let Some(e) = encoding {
				res_headers.insert("Content-Encoding", HeaderValue::from_static(e));
			}

			res
		},
		Err(_e) => {
			//try index
			handler
				.get_part("index.html", Some("html"))
				.await
				.unwrap_or_else(|e| Into::<ServerCoreError>::into(e).into_response())
		},
	}
}
