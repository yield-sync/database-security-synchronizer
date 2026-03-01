use std::sync::Arc;
use sqlx::mysql::MySqlRow;

use super::database_connection::DatabaseConnection;


pub struct TableSecurityExchangeTicker
{
	db_connection: Arc<DatabaseConnection>,
}

impl TableSecurityExchangeTicker
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn read_rows(&self, security_cik: &str) -> Result<Vec<MySqlRow>, Box<dyn std::error::Error>>
	{
		let existing_rows = sqlx::query("SELECT * FROM security_exchange_ticker WHERE security_cik = ?").bind(
			security_cik
		).fetch_all(
			self.db_connection.pool()
		).await?;

		Ok(existing_rows)
	}

	pub async fn find_rows(
		&self,
		security_cik: &str,
		exchange: &str,
		ticker: &str,
	) -> Result<Vec<MySqlRow>, Box<dyn std::error::Error>>
	{
		let existing_rows = sqlx::query(
			"SELECT * FROM security_exchange_ticker WHERE security_cik = ? AND exchange = ? AND ticker = ?"
		).bind(
			security_cik
		).bind(
			exchange
		).bind(
			ticker
		).fetch_all(
			self.db_connection.pool()
		).await?;

		Ok(existing_rows)
	}

	pub async fn create_row(
		&self,
		security_cik: &str,
		exchange: &str,
		ticker: &str,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query("INSERT INTO security_exchange_ticker (security_cik, exchange, ticker) VALUES (?, ?, ?)").bind(
			security_cik
		).bind(
			exchange
		).bind(
			ticker
		).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}


	pub async fn delete_row(&self, security_exchange_ticker_id: i64) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query("DELETE FROM security_exchange_ticker WHERE id = ?").bind(security_exchange_ticker_id).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}
}
