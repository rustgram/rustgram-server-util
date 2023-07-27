use std::future::Future;

use mysql_common::params::Params;
use mysql_common::prelude::FromRow;

use crate::db::TransactionData;
use crate::error::ServerCoreError;
use crate::static_var::db::db;

pub fn query<T, P>(sql: &'static str, params: P) -> impl Future<Output = Result<Vec<T>, ServerCoreError>>
where
	T: FromRow + Send + 'static,
	P: Into<Params> + Send,
{
	db().query(sql, params)
}

pub fn query_string<T, P>(sql: String, params: P) -> impl Future<Output = Result<Vec<T>, ServerCoreError>>
where
	T: FromRow + Send + 'static,
	P: Into<Params> + Send,
{
	db().query_string(sql, params)
}

pub fn query_first<T, P>(sql: &'static str, params: P) -> impl Future<Output = Result<Option<T>, ServerCoreError>>
where
	T: FromRow + Send + 'static,
	P: Into<Params> + Send,
{
	db().query_first(sql, params)
}

pub fn query_first_string<T, P>(sql: String, params: P) -> impl Future<Output = Result<Option<T>, ServerCoreError>>
where
	T: FromRow + Send + 'static,
	P: Into<Params> + Send,
{
	db().query_first_string(sql, params)
}

pub fn query_non_param<T>(sql: &'static str) -> impl Future<Output = Result<Vec<T>, ServerCoreError>>
where
	T: FromRow + Send + 'static,
{
	db().query_non_param(sql)
}

pub fn query_string_non_param<T>(sql: String) -> impl Future<Output = Result<Vec<T>, ServerCoreError>>
where
	T: FromRow + Send + 'static,
{
	db().query_string_non_param(sql)
}

pub fn query_first_non_param<T>(sql: &'static str) -> impl Future<Output = Result<Option<T>, ServerCoreError>>
where
	T: FromRow + Send + 'static,
{
	db().query_first_non_param(sql)
}

pub fn query_first_string_non_param<T>(sql: String) -> impl Future<Output = Result<Option<T>, ServerCoreError>>
where
	T: FromRow + Send + 'static,
{
	db().query_first_string_non_param(sql)
}

pub fn exec<P>(sql: &'static str, params: P) -> impl Future<Output = Result<(), ServerCoreError>>
where
	P: Into<Params> + Send,
{
	db().exec(sql, params)
}

pub fn exec_string<P>(sql: String, params: P) -> impl Future<Output = Result<(), ServerCoreError>>
where
	P: Into<Params> + Send,
{
	db().exec_string(sql, params)
}

pub fn exec_non_param(sql: &'static str) -> impl Future<Output = Result<(), ServerCoreError>>
{
	db().exec_non_param(sql)
}

pub fn exec_string_non_param(sql: String) -> impl Future<Output = Result<(), ServerCoreError>>
{
	db().exec_string_non_param(sql)
}

pub fn exec_transaction<'a, P>(data: Vec<TransactionData<'a, P>>) -> impl Future<Output = Result<(), ServerCoreError>> + 'a
where
	P: Into<Params> + Send + 'a,
{
	db().exec_transaction(data)
}

pub fn bulk_insert<'a, F, P, T>(
	ignore: bool,
	table: &'a str,
	cols: &'a [&'a str],
	objects: Vec<T>,
	fun: F,
) -> impl Future<Output = Result<(), ServerCoreError>> + 'a
where
	T: 'a,
	F: Fn(T) -> P + 'a,
	P: Into<Params> + 'a,
{
	db().bulk_insert(ignore, table, cols, objects, fun)
}
