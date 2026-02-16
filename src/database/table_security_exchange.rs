use std::sync::Arc;
use super::database_connection::DatabaseConnection;


pub struct TableSecurityExchange
{
	db_connection: Arc<DatabaseConnection>,
}

impl TableSecurityExchange
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn create_row(
		&self,
		security_ticker_id: &i64,
		exchange: &str,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query("INSERT INTO security_exchange (security_ticker_id, exchange) VALUES (?, ?)").bind(
			security_ticker_id
		).bind(
			exchange
		).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}

	pub async fn create_rows(
		&self,
		security_ticker_ids: &Vec<i64>,
		exchanges: &Vec<String>,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		let strs_exchange: Vec<&str> = exchanges.iter().map(String::as_str).collect();

		if security_ticker_ids.len() != strs_exchange.len()
		{
			return Err("security_ticker_id and exchanges must have the same length".into());
		}

		for (security_ticker_id, exchange) in security_ticker_ids.iter().zip(strs_exchange.iter())
		{
			self.create_row(&security_ticker_id, &exchange).await?;
		}

		Ok(())
	}


	pub async fn delete_row(&self, security_ticker_id: i64) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query("DELETE FROM security_exchange WHERE security_ticker_id = ?").bind(security_ticker_id).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}
}
