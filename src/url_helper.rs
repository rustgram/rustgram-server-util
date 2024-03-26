use std::collections::HashMap;
use std::str::FromStr;

use rustgram::{Request, RouteParams};

use crate::error::{server_err, CoreErrorCodes};
use crate::res::AppRes;

pub fn get_params(req: &Request) -> AppRes<&RouteParams>
{
	match req.extensions().get::<RouteParams>() {
		Some(p) => Ok(p),
		None => Err(server_err(400, CoreErrorCodes::NoParameter, "No parameter sent")),
	}
}

pub fn get_name_param_from_req<'a>(req: &'a Request, name: &str) -> AppRes<&'a str>
{
	let params = get_params(req)?;

	match params.get(name) {
		None => Err(server_err(400, CoreErrorCodes::NoParameter, "Parameter not found")),
		Some(n) => Ok(n),
	}
}

pub fn get_name_param_from_params<'a>(params: &'a RouteParams, name: &str) -> AppRes<&'a str>
{
	//this is useful if we need more than one params, so we don't need to get it from req multiple times
	match params.get(name) {
		None => Err(server_err(400, CoreErrorCodes::NoParameter, "Parameter not found")),
		Some(n) => Ok(n),
	}
}

pub fn get_query_params(req: &Request) -> AppRes<HashMap<String, String>>
{
	let query = match req.uri().query() {
		Some(q) => q,
		None => {
			return Err(server_err(400, CoreErrorCodes::NoUrlQuery, "Url query not found"));
		},
	};

	let params: HashMap<String, String> = query
		.split('&')
		.map(|p| p.split('=').map(|s| s.to_string()).collect::<Vec<String>>())
		.filter(|p| p.len() == 2)
		.map(|p| (p[0].clone(), p[1].clone()))
		.collect();

	Ok(params)
}

pub fn get_number_from_url_param<T: FromStr>(number: &str) -> AppRes<T>
{
	number
		.parse()
		.map_err(|_e| server_err(400, CoreErrorCodes::UnexpectedTime, "It must be a number"))
}

pub fn get_time_from_url_param(time: &str) -> AppRes<u128>
{
	get_number_from_url_param(time)
}
