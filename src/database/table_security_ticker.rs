use std::sync::Arc;
use sqlx::mysql::MySqlRow;


use super::database_connection::DatabaseConnection;


pub struct TableSecurityTicker
{
	db_connection: Arc<DatabaseConnection>,
}

impl TableSecurityTicker
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn read_rows(&self, security_cik: &str) -> Result<Vec<MySqlRow>, Box<dyn std::error::Error>>
	{
		let existing_rows = sqlx::query("SELECT * FROM security_ticker WHERE security_cik = ?").bind(
			security_cik
		).fetch_all(
			self.db_connection.pool()
		).await?;

		Ok(existing_rows)
	}

	pub async fn create_row(&self, security_cik: &str, ticker: &str) -> Result<i64, Box<dyn std::error::Error>>
	{
		let result = sqlx::query("INSERT INTO security_ticker (security_cik, ticker) VALUES (?, ?)").bind(
			security_cik
		).bind(
			ticker
		).execute(
			self.db_connection.pool()
		).await?;

		let id = result.last_insert_id() as i64;

		Ok(id)
	}

	pub async fn delete_row(&self, id: i64) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query("DELETE FROM security_ticker WHERE id = ?").bind(id).execute(self.db_connection.pool()).await?;

		Ok(())
	}
}
