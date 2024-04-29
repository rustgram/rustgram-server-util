use std::error::Error;
use std::fmt::{Display, Formatter};

use deadpool_sqlite::{Config, Pool, Runtime};
use rusqlite::{params_from_iter, Connection, Row, ToSql};

use crate::db::{db_bulk_insert_err, db_exec_err, db_query_err, db_tx_err};
use crate::error::{CoreErrorCodes, ServerCoreError, ServerErrorConstructor};

#[macro_export]
macro_rules! take_or_err {
	($row:expr, $index:expr) => {
		match $row.get($index) {
			Ok(v) => v,
			Err(e) => {
				return Err($crate::db::FormSqliteRowError {
					msg: format!("{:?}", e),
				})
			},
		}
	};
}

#[macro_export]
macro_rules! take_or_err_u128 {
	($row:expr, $index:expr) => {
		match $row.get($index) {
			Ok(v) => {
				let str: String = v;
				let str: u128 = match str.parse() {
					Ok(v) => v,
					Err(e) => {
						return Err($crate::db::FormSqliteRowError {
							msg: format!("err in db fetch: {:?}", e),
						})
					},
				};
				str
			},
			Err(e) => {
				return Err($crate::db::FormSqliteRowError {
					msg: format!("{:?}", e),
				})
			},
		}
	};
}

#[macro_export]
macro_rules! take_or_err_usize {
	($row:expr, $index:expr) => {
		match $row.get($index) {
			Ok(v) => {
				let str: String = v;
				let str: usize = match str.parse() {
					Ok(v) => v,
					Err(e) => {
						return Err($crate::db::FormSqliteRowError {
							msg: format!("err in db fetch: {:?}", e),
						})
					},
				};
				str
			},
			Err(e) => {
				return Err($crate::db::FormSqliteRowError {
					msg: format!("{:?}", e),
				})
			},
		}
	};
}

pub struct TransactionData<P>
where
	P: IntoIterator,
	P::Item: ToSql,
{
	pub sql: &'static str,
	pub params: P,
}

#[derive(Debug)]
pub struct FormSqliteRowError
{
	pub msg: String,
}

impl Error for FormSqliteRowError {}

impl Display for FormSqliteRowError
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
	{
		write!(f, "Err in db fetch: {}", self.msg)
	}
}

pub trait FromSqliteRow
{
	fn from_row_opt(row: &Row) -> Result<Self, FormSqliteRowError>
	where
		Self: Sized;
}

pub struct Sqlite
{
	pool: Pool,
}

impl Sqlite
{
	pub fn new(path: &str) -> Self
	{
		let cfg = Config::new(path);

		let pool = cfg.create_pool(Runtime::Tokio1).unwrap();

		#[cfg(debug_assertions)]
		println!("init sqlite");

		Self {
			pool,
		}
	}

	pub fn new_with_config(cfg: Config) -> Self
	{
		#[cfg(debug_assertions)]
		println!("init sqlite");

		Self {
			pool: cfg.create_pool(Runtime::Tokio1).unwrap(),
		}
	}

	async fn get_conn(&self) -> Result<deadpool_sqlite::Object, ServerCoreError>
	{
		self.pool.get().await.map_err(|e| {
			ServerCoreError::new_msg_and_debug(
				500,
				CoreErrorCodes::NoDbConnection,
				"No db connection",
				Some(format!("db connection error for sqlite: {:?}", e)),
			)
		})
	}

	/**
	# Execute and fetch from db

	````ignore
	use rusqlite::Row;

	pub struct Lol
	{
		pub lol: String,
		pub lol_count: i32,
	}

	impl FromSqliteRow for Lol
	{
		fn from_row_opt(row: &Row) -> Result<Self, FormSqliteRowError>
		where
			Self: Sized,
		{
			Ok(Lol {
				lol: take_or_err(row, 0),
				lol_count: take_or_err(row, 1,
			})
		}
	}


	async fn lol()
	{
		//language=SQL
		let sql = "SELECT 1";
		let params = crate::set_params!("1".to_string(), 2_i32);

		let lol = query::<Lol, _>(sql, params).await.unwrap();

		//or from a vec (every item must be the same type for vec

		let param_vec = vec!["123".to_string(), "1".to_string()];

		let lol = query::<Lol, _>(sql, param_vec).await.unwrap();
	}

	````
	 */
	pub async fn query<T, P>(&self, sql: &'static str, params: P) -> Result<Vec<T>, ServerCoreError>
	where
		T: FromSqliteRow + Send + 'static,
		P: IntoIterator + Send + 'static,
		P::Item: ToSql,
	{
		let conn = self.get_conn().await?;

		let result = conn
			.interact(move |conn| query_sync::<T, P>(conn, sql, params))
			.await
			.map_err(|e| db_query_err(&e, "Interact Error"))??;

		Ok(result)
	}

	/**
	The same as query but sql with a string.

	This is used to get the sql string from the get in fn
	 */
	pub async fn query_string<T, P>(&self, sql: String, params: P) -> Result<Vec<T>, ServerCoreError>
	where
		T: FromSqliteRow + Send + 'static,
		P: IntoIterator + Send + 'static,
		P::Item: ToSql,
	{
		let conn = self.get_conn().await?;

		let result = conn
			.interact(move |conn| query_sync::<T, P>(conn, sql.as_str(), params))
			.await
			.map_err(|e| db_query_err(&e, "Interact Error"))??;

		Ok(result)
	}

	/**
	# Query and get the first result

	No vec gets returned, but an options enum
	 */
	pub async fn query_first<T, P>(&self, sql: &'static str, params: P) -> Result<Option<T>, ServerCoreError>
	where
		T: FromSqliteRow + Send + 'static,
		P: IntoIterator + Send + 'static,
		P::Item: ToSql,
	{
		let conn = self.get_conn().await?;

		let result = conn
			.interact(move |conn| query_first_sync::<T, P>(conn, sql, params))
			.await
			.map_err(|e| db_query_err(&e, "Interact Error"))??;

		Ok(result)
	}

	/**
	The same as query but sql with a string.

	This is used to get the sql string from the get in fn
	 */
	pub async fn query_first_string<T, P>(&self, sql: String, params: P) -> Result<Option<T>, ServerCoreError>
	where
		T: FromSqliteRow + Send + 'static,
		P: IntoIterator + Send + 'static,
		P::Item: ToSql,
	{
		let conn = self.get_conn().await?;

		let result = conn
			.interact(move |conn| query_first_sync::<T, P>(conn, sql.as_str(), params))
			.await
			.map_err(|e| db_query_err(&e, "Interact Error"))??;

		Ok(result)
	}

	pub async fn query_non_param<T>(&self, sql: &'static str) -> Result<Vec<T>, ServerCoreError>
	where
		T: FromSqliteRow + Send + 'static,
	{
		let conn = self.get_conn().await?;

		let result = conn
			.interact(move |conn| query_non_param_sync(conn, sql))
			.await
			.map_err(|e| db_query_err(&e, "Interact Error"))??;

		Ok(result)
	}

	pub async fn query_string_non_param<T>(&self, sql: String) -> Result<Vec<T>, ServerCoreError>
	where
		T: FromSqliteRow + Send + 'static,
	{
		let conn = self.get_conn().await?;

		let result = conn
			.interact(move |conn| query_non_param_sync(conn, sql.as_str()))
			.await
			.map_err(|e| db_query_err(&e, "Interact Error"))??;

		Ok(result)
	}

	pub async fn query_first_non_param<T>(&self, sql: &'static str) -> Result<Option<T>, ServerCoreError>
	where
		T: FromSqliteRow + Send + 'static,
	{
		let conn = self.get_conn().await?;

		let result = conn
			.interact(move |conn| query_first_non_param_sync(conn, sql))
			.await
			.map_err(|e| db_query_err(&e, "Interact Error"))??;

		Ok(result)
	}

	pub async fn query_first_string_non_param<T>(&self, sql: String) -> Result<Option<T>, ServerCoreError>
	where
		T: FromSqliteRow + Send + 'static,
	{
		let conn = self.get_conn().await?;

		let result = conn
			.interact(move |conn| query_first_non_param_sync(conn, sql.as_str()))
			.await
			.map_err(|e| db_query_err(&e, "Interact Error"))??;

		Ok(result)
	}

	/**
	# Executes an sql stmt

	````ignore
	async fn lol()
	{
		//language=SQL
		let sql = "INSERT INTO table (col1, col2) VALUES (?,?)";
		let params = crate::set_params!("1".to_string(), 2_i32);

		let lol = exec(sql, params).await.unwrap();

		//or from a vec (every item must be the same type for vec

		let param_vec = vec!["123".to_string(), "1".to_string()];

		let lol = exec(sql, param_vec).await.unwrap();
	}

	````
	 */
	pub async fn exec<P>(&self, sql: &'static str, params: P) -> Result<(), ServerCoreError>
	where
		P: IntoIterator + Send + 'static,
		P::Item: ToSql,
	{
		let conn = self.get_conn().await?;

		conn.interact(move |conn| exec_sync(conn, sql, params))
			.await
			.map_err(|e| db_exec_err(&e, "Interact Error"))??;

		Ok(())
	}

	pub async fn exec_string<P>(&self, sql: String, params: P) -> Result<(), ServerCoreError>
	where
		P: IntoIterator + Send + 'static,
		P::Item: ToSql,
	{
		let conn = self.get_conn().await?;

		conn.interact(move |conn| exec_sync(conn, sql.as_str(), params))
			.await
			.map_err(|e| db_exec_err(&e, "Interact Error"))??;

		Ok(())
	}

	pub async fn exec_non_param(&self, sql: &'static str) -> Result<(), ServerCoreError>
	{
		let conn = self.get_conn().await?;

		conn.interact(move |conn| exec_non_param_sync(conn, sql))
			.await
			.map_err(|e| db_exec_err(&e, "Interact Error"))??;

		Ok(())
	}

	pub async fn exec_string_non_param(&self, sql: String) -> Result<(), ServerCoreError>
	{
		let conn = self.get_conn().await?;

		conn.interact(move |conn| exec_non_param_sync(conn, sql.as_str()))
			.await
			.map_err(|e| db_exec_err(&e, "Interact Error"))??;

		Ok(())
	}

	/**
	# Execute in transaction

	can be multiple stmt with params in one transition
	 */
	pub async fn exec_transaction<P>(&self, data: Vec<TransactionData<P>>) -> Result<(), ServerCoreError>
	where
		P: IntoIterator + Send + 'static,
		P::Item: ToSql,
	{
		let conn = self.get_conn().await?;

		conn.interact(move |conn| exec_transaction_sync(conn, data))
			.await
			.map_err(|e| db_exec_err(&e, "Interact Error"))?
	}

	/**
	# let insert multiple objets into the db

	got it form here: https://github.com/blackbeam/rust-mysql-simple/issues/59#issuecomment-245918807

	`T` is the object type

	`fn` transformed the obj values to params

	`ignore` do an insert ignore

	creates a query like this:
	```SQL
	INSERT INTO table (fields...) VALUES (?, ?, ?), (?, ?, ?), (?, ?, ?), ...
	```
	 */
	pub async fn bulk_insert<F: 'static + Send + Sync, T: 'static + Send + Sync>(
		&self,
		ignore: bool,
		table: &'static str,
		cols: &'static [&'static str],
		objects: Vec<T>, //must be pass by value because we need static lifetime here for the deadpool interact
		fun: F,
	) -> Result<(), ServerCoreError>
	where
		F: Fn(T) -> Vec<rusqlite::types::Value>,
	{
		let conn = self.get_conn().await?;

		let _res = conn
			.interact(move |conn| bulk_insert_sync(conn, ignore, table, cols, objects, fun))
			.await
			.map_err(|e| db_bulk_insert_err(&e, table))??;

		Ok(())
	}
}

fn query_sync<T, P>(conn: &mut Connection, sql: &str, params: P) -> Result<Vec<T>, ServerCoreError>
where
	T: FromSqliteRow,
	P: IntoIterator,
	P::Item: ToSql,
{
	let mut stmt = conn.prepare(sql).map_err(|e| db_query_err(&e, sql))?;

	let mut rows = stmt
		.query(params_from_iter(params))
		.map_err(|e| db_query_err(&e, sql))?;

	let mut init: Vec<T> = Vec::new();

	while let Some(row) = rows.next().map_err(|e| db_query_err(&e, sql))? {
		init.push(FromSqliteRow::from_row_opt(row).map_err(|e| db_query_err(&e, sql))?)
	}

	Ok(init)
}

fn query_first_sync<T, P>(conn: &mut Connection, sql: &str, params: P) -> Result<Option<T>, ServerCoreError>
where
	T: FromSqliteRow,
	P: IntoIterator,
	P::Item: ToSql,
{
	let mut stmt = conn.prepare(sql).map_err(|e| db_query_err(&e, sql))?;

	let mut rows = stmt
		.query(params_from_iter(params))
		.map_err(|e| db_query_err(&e, sql))?;

	match rows.next().map_err(|e| db_query_err(&e, sql))? {
		Some(row) => {
			Ok(Some(
				FromSqliteRow::from_row_opt(row).map_err(|e| db_query_err(&e, sql))?,
			))
		},
		None => Ok(None),
	}
}

fn query_non_param_sync<T>(conn: &mut Connection, sql: &str) -> Result<Vec<T>, ServerCoreError>
where
	T: FromSqliteRow,
{
	let mut stmt = conn.prepare(sql).map_err(|e| db_query_err(&e, sql))?;

	let mut rows = stmt.query([]).map_err(|e| db_query_err(&e, sql))?;

	let mut init: Vec<T> = Vec::new();

	while let Some(row) = rows.next().map_err(|e| db_query_err(&e, sql))? {
		init.push(FromSqliteRow::from_row_opt(row).map_err(|e| db_query_err(&e, sql))?)
	}

	Ok(init)
}

fn query_first_non_param_sync<T>(conn: &mut Connection, sql: &str) -> Result<Option<T>, ServerCoreError>
where
	T: FromSqliteRow,
{
	let mut stmt = conn.prepare(sql).map_err(|e| db_query_err(&e, sql))?;

	let mut rows = stmt.query([]).map_err(|e| db_query_err(&e, sql))?;

	match rows.next().map_err(|e| db_query_err(&e, sql))? {
		Some(row) => {
			Ok(Some(
				FromSqliteRow::from_row_opt(row).map_err(|e| db_query_err(&e, sql))?,
			))
		},
		None => Ok(None),
	}
}

fn exec_sync<P>(conn: &mut Connection, sql: &str, params: P) -> Result<usize, ServerCoreError>
where
	P: IntoIterator,
	P::Item: ToSql,
{
	conn.execute(sql, params_from_iter(params))
		.map_err(|e| db_exec_err(&e, sql))
}

fn exec_non_param_sync(conn: &mut Connection, sql: &str) -> Result<usize, ServerCoreError>
{
	conn.execute(sql, []).map_err(|e| db_exec_err(&e, sql))
}

fn exec_transaction_sync<P>(conn: &mut Connection, data: Vec<TransactionData<P>>) -> Result<(), ServerCoreError>
where
	P: IntoIterator,
	P::Item: ToSql,
{
	let tx = conn.transaction().map_err(|e| db_tx_err(&e))?;

	for datum in data {
		tx.execute(datum.sql, params_from_iter(datum.params))
			.map_err(|e| db_tx_err(&e))?;
	}

	tx.commit().map_err(|e| db_tx_err(&e))
}

fn bulk_insert_sync<F, T>(conn: &mut Connection, ignore: bool, table: &str, cols: &[&str], objects: Vec<T>, fun: F) -> Result<usize, ServerCoreError>
where
	F: Fn(T) -> Vec<rusqlite::types::Value>,
{
	//prepare the sql
	let ignore_string = if ignore { " OR IGNORE" } else { "" };

	let mut stmt = format!("INSERT {} INTO {} ({}) VALUES ", ignore_string, table, cols.join(","));
	// each (?,..,?) tuple for values
	let row = format!("({}),", cols.iter().map(|_| "?").collect::<Vec<_>>().join(","));

	stmt.reserve(objects.len() * (cols.len() * 2 + 2));

	// add the row tuples in the query
	for _ in 0..objects.len() {
		stmt.push_str(&row);
	}

	// remove the trailing comma
	stmt.pop();

	let mut params = Vec::new();

	//using rustsqlite value https://stackoverflow.com/questions/69230495/how-to-pass-vecvalue-in-rusqlite-as-query-param
	for o in objects {
		for val in fun(o) {
			params.push(val);
		}
	}

	//transaction from here: https://github.com/avinassh/fast-sqlite3-inserts/blob/master/src/bin/basic.rs
	//but not necessary for inserting in one table
	let tx = conn
		.transaction()
		.map_err(|e| db_bulk_insert_err(&e, table))?;

	let result = tx
		.execute(stmt.as_str(), params_from_iter(params))
		.map_err(|e| db_bulk_insert_err(&e, table))?;

	tx.commit().map_err(|e| db_bulk_insert_err(&e, table))?;

	Ok(result)
}
