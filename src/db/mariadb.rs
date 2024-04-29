use mysql_async::prelude::{FromRow, Queryable};
use mysql_async::{OptsBuilder, Params, Pool, TxOpts};

use crate::db::{db_bulk_insert_err, db_exec_err, db_query_err, db_tx_err};
use crate::error::{CoreErrorCodes, ServerCoreError, ServerErrorConstructor};

#[macro_export]
macro_rules! take_or_err {
	($row:expr, $index:expr, $t:ident) => {
		match $row.take_opt::<$t, _>($index) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err($crate::db::mysql_async_export::FromValueError(_value)) => {
						return Err($crate::db::mysql_async_export::FromRowError($row));
					},
				}
			},
			None => return Err($crate::db::mysql_async_export::FromRowError($row)),
		}
	};
	($row:expr, $index:expr, Option<$t:ident>) => {
		match $row.take_opt::<Option<$t>, _>($index) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err($crate::db::mysql_async_export::FromValueError(_value)) => {
						return Err($crate::db::mysql_async_export::FromRowError($row));
					},
				}
			},
			None => return Err($crate::db::mysql_async_export::FromRowError($row)),
		}
	};
}

#[macro_export]
macro_rules! take_or_err_opt {
	($row:expr, $index:expr, $t:ident) => {
		match $row.take_opt::<Option<$t>, _>($index) {
			Some(value) => {
				match value {
					Ok(ir) => ir,
					Err($crate::db::mysql_async_export::FromValueError(_value)) => {
						return Err($crate::db::mysql_async_export::FromRowError($row));
					},
				}
			},
			None => return Err($crate::db::mysql_async_export::FromRowError($row)),
		}
	};
}

pub struct TransactionData<'a, P>
where
	P: Into<Params> + Send,
{
	pub sql: &'a str,
	pub params: P,
}

pub struct Mariadb
{
	pool: Pool,
}

impl Mariadb
{
	pub fn new(user: &str, pw: &str, mysql_host: &str, db_name: &str, db_port: Option<u16>) -> Self
	{
		#[cfg(debug_assertions)]
		println!("init mariadb");

		let opts = if let Some(port) = db_port {
			OptsBuilder::default()
				.ip_or_hostname(mysql_host)
				.db_name(Some(db_name))
				.user(Some(user))
				.pass(Some(pw))
				.tcp_port(port)
		} else {
			OptsBuilder::default()
				.ip_or_hostname(mysql_host)
				.db_name(Some(db_name))
				.user(Some(user))
				.pass(Some(pw))
		};

		Self {
			pool: Pool::new(opts),
		}
	}

	pub fn new_with_conn_str(str: &str) -> Self
	{
		#[cfg(debug_assertions)]
		println!("init mariadb");

		Self {
			pool: Pool::new(str),
		}
	}

	pub fn new_with_pool(pool: Pool) -> Self
	{
		#[cfg(debug_assertions)]
		println!("init mariadb");

		Self {
			pool,
		}
	}

	async fn get_conn(&self) -> Result<mysql_async::Conn, ServerCoreError>
	{
		//get conn with a loop because for very much workload we getting an err -> try again

		let mut i = 0; //say how much iteration should be done before giving up

		loop {
			if i > 10 {
				return Err(ServerCoreError::new_msg_and_debug(
					500,
					CoreErrorCodes::NoDbConnection,
					"No db connection",
					Some("No connection after 10 tries".to_owned()),
				));
			}

			match self.pool.get_conn().await {
				Ok(conn_ty) => {
					return Ok(conn_ty);
				},
				Err(_e) => {
					//println!("{:?}", e);
				},
			}

			i += 1;
		}
	}

	/**
	# call mysql-async exec function

	handles the err and return a `HttpErr` instead of the db err

	so we can just use it like:
	```ignore
	//language=SQL
	let sql = "SELECT tag_id, belongs_to, type FROM tags_belongs_to WHERE tag_id = ?";

	// the , in ("lol",) is important!
	//exec is from mysql_async
	let result = exec::<TagsBelongsTo, _>(sql, ("lol",)).await?;

	match to_string(&result) {
		Ok(o) => Ok(o),
		Err(e) => Err(HttpErr::new(422, 10, format!("db error"), Some(format!("db fetch err, {:?}", e)))),
	}
	```

	instead of this (don't do this, no err handling here):
	```ignore
	//language=SQL
	let sql = "SELECT tag_id, belongs_to, type FROM tags_belongs_to WHERE tag_id = ?";

	let mut conn = get_conn().await?;

	// the , in ("lol",) is important!
	let result = conn
		.query::<TagsBelongsTo, _, _>(sql, ("lol",))
		.await
		.unwrap();

	Ok(to_string(&result).unwrap())
	```
	 */
	pub async fn query<T, P>(&self, sql: &'static str, params: P) -> Result<Vec<T>, ServerCoreError>
	where
		T: FromRow + Send + 'static,
		P: Into<Params> + Send,
	{
		let mut conn = self.get_conn().await?;

		conn.exec::<T, _, P>(sql, params)
			.await
			.map_err(|e| db_query_err(&e, sql))
	}

	/**
	The same as query but sql with a string.

	This is used to get the sql string from the get in fn
	 */
	pub async fn query_string<T, P>(&self, sql: String, params: P) -> Result<Vec<T>, ServerCoreError>
	where
		T: FromRow + Send + 'static,
		P: Into<Params> + Send,
	{
		let mut conn = self.get_conn().await?;

		conn.exec::<T, _, P>(sql, params)
			.await
			.map_err(|e| db_query_err(&e, ""))
	}

	/**
	# Query and get the first result

	No vec gets returned, but an options enum
	 */
	pub async fn query_first<T, P>(&self, sql: &'static str, params: P) -> Result<Option<T>, ServerCoreError>
	where
		T: FromRow + Send + 'static,
		P: Into<Params> + Send,
	{
		let mut conn = self.get_conn().await?;

		conn.exec_first::<T, _, P>(sql, params)
			.await
			.map_err(|e| db_query_err(&e, sql))
	}

	/**
	The same as query but sql with a string.

	This is used to get the sql string from the get in fn
	 */
	pub async fn query_first_string<T, P>(&self, sql: String, params: P) -> Result<Option<T>, ServerCoreError>
	where
		T: FromRow + Send + 'static,
		P: Into<Params> + Send,
	{
		let mut conn = self.get_conn().await?;

		conn.exec_first::<T, _, P>(sql, params)
			.await
			.map_err(|e| db_query_err(&e, ""))
	}

	pub async fn query_non_param<T>(&self, sql: &'static str) -> Result<Vec<T>, ServerCoreError>
	where
		T: FromRow + Send + 'static,
	{
		let mut conn = self.get_conn().await?;

		conn.query(sql).await.map_err(|e| db_query_err(&e, sql))
	}

	pub async fn query_string_non_param<T>(&self, sql: String) -> Result<Vec<T>, ServerCoreError>
	where
		T: FromRow + Send + 'static,
	{
		let mut conn = self.get_conn().await?;

		conn.query(sql).await.map_err(|e| db_query_err(&e, ""))
	}

	pub async fn query_first_non_param<T>(&self, sql: &'static str) -> Result<Option<T>, ServerCoreError>
	where
		T: FromRow + Send + 'static,
	{
		let mut conn = self.get_conn().await?;

		conn.query_first(sql)
			.await
			.map_err(|e| db_query_err(&e, sql))
	}

	pub async fn query_first_string_non_param<T>(&self, sql: String) -> Result<Option<T>, ServerCoreError>
	where
		T: FromRow + Send + 'static,
	{
		let mut conn = self.get_conn().await?;

		conn.query_first(sql)
			.await
			.map_err(|e| db_query_err(&e, ""))
	}

	/**
	# Execute a sql stmt

	drop the result just execute
	 */
	pub async fn exec<P>(&self, sql: &str, params: P) -> Result<(), ServerCoreError>
	where
		P: Into<Params> + Send,
	{
		let mut conn = self.get_conn().await?;

		conn.exec_drop(sql, params)
			.await
			.map_err(|e| db_exec_err(&e, sql))
	}

	pub async fn exec_string<P>(&self, sql: String, params: P) -> Result<(), ServerCoreError>
	where
		P: Into<Params> + Send,
	{
		let mut conn = self.get_conn().await?;

		conn.exec_drop(sql, params)
			.await
			.map_err(|e| db_exec_err(&e, ""))
	}

	pub async fn exec_non_param(&self, sql: &str) -> Result<(), ServerCoreError>
	{
		let mut conn = self.get_conn().await?;

		conn.query_drop(sql).await.map_err(|e| db_exec_err(&e, sql))
	}

	pub async fn exec_string_non_param(&self, sql: String) -> Result<(), ServerCoreError>
	{
		let mut conn = self.get_conn().await?;

		conn.query_drop(sql).await.map_err(|e| db_exec_err(&e, ""))
	}

	/**
	# Execute in transaction

	can be multiple stmt with params in one transition
	 */
	pub async fn exec_transaction<P>(&self, data: Vec<TransactionData<'_, P>>) -> Result<(), ServerCoreError>
	where
		P: Into<Params> + Send,
	{
		let mut conn = self.get_conn().await?;
		let mut tx = conn
			.start_transaction(TxOpts::default())
			.await
			.map_err(|e| db_tx_err(&e))?;

		for datum in data {
			tx.exec_drop(datum.sql, datum.params)
				.await
				.map_err(|e| db_tx_err(&e))?;
		}

		tx.commit().await.map_err(|e| db_tx_err(&e))
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
	pub async fn bulk_insert<F, P, T>(&self, ignore: bool, table: &str, cols: &[&str], objects: Vec<T>, fun: F) -> Result<(), ServerCoreError>
	where
		F: Fn(T) -> P,
		P: Into<Params>,
	{
		let ignore_string = if ignore { "IGNORE" } else { "" };

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

		for o in objects {
			let new_params: Params = fun(o).into();

			if let Params::Positional(new_params) = new_params {
				for param in new_params {
					params.push(param);
				}
			}
		}

		let mut conn = self.get_conn().await?;

		conn.exec_drop(stmt, params)
			.await
			.map_err(|e| db_bulk_insert_err(&e, table))
	}
}
