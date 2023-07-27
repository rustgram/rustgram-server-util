use std::future::Future;

use rusqlite::ToSql;

use crate::db::{FromSqliteRow, TransactionData};
use crate::error::ServerCoreError;
use crate::static_var::db::db;

pub fn query<T, P>(sql: &'static str, params: P) -> impl Future<Output = Result<Vec<T>, ServerCoreError>>
where
	T: FromSqliteRow + Send + 'static,
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	db().query(sql, params)
}

pub fn query_string<T, P>(sql: String, params: P) -> impl Future<Output = Result<Vec<T>, ServerCoreError>>
where
	T: FromSqliteRow + Send + 'static,
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	db().query_string(sql, params)
}

pub fn query_first<T, P>(sql: &'static str, params: P) -> impl Future<Output = Result<Option<T>, ServerCoreError>>
where
	T: FromSqliteRow + Send + 'static,
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	db().query_first(sql, params)
}

pub fn query_first_string<T, P>(sql: String, params: P) -> impl Future<Output = Result<Option<T>, ServerCoreError>>
where
	T: FromSqliteRow + Send + 'static,
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	db().query_first_string(sql, params)
}

pub fn query_non_param<T>(sql: &'static str) -> impl Future<Output = Result<Vec<T>, ServerCoreError>>
where
	T: FromSqliteRow + Send + 'static,
{
	db().query_non_param(sql)
}

pub fn query_string_non_param<T>(sql: String) -> impl Future<Output = Result<Vec<T>, ServerCoreError>>
where
	T: FromSqliteRow + Send + 'static,
{
	db().query_string_non_param(sql)
}

pub fn query_first_non_param<T>(sql: &'static str) -> impl Future<Output = Result<Option<T>, ServerCoreError>>
where
	T: FromSqliteRow + Send + 'static,
{
	db().query_first_non_param(sql)
}

pub fn query_first_string_non_param<T>(sql: String) -> impl Future<Output = Result<Option<T>, ServerCoreError>>
where
	T: FromSqliteRow + Send + 'static,
{
	db().query_first_string_non_param(sql)
}

pub fn exec<P>(sql: &'static str, params: P) -> impl Future<Output = Result<(), ServerCoreError>>
where
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	db().exec(sql, params)
}

pub fn exec_string<P>(sql: String, params: P) -> impl Future<Output = Result<(), ServerCoreError>>
where
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
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

pub fn exec_transaction<P>(data: Vec<TransactionData<P>>) -> impl Future<Output = Result<(), ServerCoreError>>
where
	P: IntoIterator + Send + 'static,
	P::Item: ToSql,
{
	db().exec_transaction(data)
}

pub fn bulk_insert<F: 'static + Send + Sync, T: 'static + Send + Sync>(
	ignore: bool,
	table: &'static str,
	cols: &'static [&'static str],
	objects: Vec<T>, //must be pass by value because we need static lifetime here for the deadpool interact
	fun: F,
) -> impl Future<Output = Result<(), ServerCoreError>>
where
	F: Fn(T) -> Vec<rusqlite::types::Value>,
{
	db().bulk_insert(ignore, table, cols, objects, fun)
}
