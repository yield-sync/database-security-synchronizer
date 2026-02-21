use std::collections::HashSet;
use std::sync::Arc;
use sqlx::Row;
use sqlx::mysql::MySqlRow;

use crate::database::database_connection::DatabaseConnection;
use crate::database::table_security_exchange::TableSecurityExchange;
use crate::database::table_security_ticker::TableSecurityTicker;

use crate::{ log_info };


pub struct HandlerSecurityExchangeTicker
{
	t_security_exchange: TableSecurityExchange,
	t_security_ticker: TableSecurityTicker
}


impl HandlerSecurityExchangeTicker
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			t_security_exchange: TableSecurityExchange::new(db_connection.clone()),
			t_security_ticker: TableSecurityTicker::new(db_connection.clone()),
		}
	}

	pub async fn synchronize(
		&self,
		security_cik: &str,
		tickers: &Vec<String>,
		exchanges: &Vec<String>,
	) -> Result<Vec<i64>, Box<dyn std::error::Error>>
	{
		// Create a set of provided_tickers to be synchronized too
		let provided_tickers: HashSet<&str> = tickers.iter().map(String::as_str).collect();

		let mut existing_tickers: HashSet<String> = HashSet::new();

		let mut existing_tickers_to_id = std::collections::HashMap::new();

		// 1. Fetch existing tickers
		let existing_rows: Vec<MySqlRow> = self.t_security_ticker.read_rows(security_cik).await?;

		log_info!("Synchronizing security tickers..");

		for row in existing_rows
		{
			let ticker: String = row.get("ticker");

			let id: i64 = row.get("id");

			existing_tickers.insert(ticker.clone());

			existing_tickers_to_id.insert(ticker, id);
		}

		// 2a. Determine what needs to be inserted
		let to_be_inserted: Vec<&str> = provided_tickers.iter().filter(
			|t| !existing_tickers.contains(**t)
		).copied().collect();

		// 2b. Insert tickers
		let mut inserted_ids = Vec::new();

		for ticker in to_be_inserted
		{
			inserted_ids.push(self.t_security_ticker.create_row(security_cik, ticker).await?);
		}

		log_info!("Synchronizing security exchanges..");

		// 2c. Insert exchanges for tickers
		self.t_security_exchange.create_rows(&inserted_ids, &exchanges).await?;

		// 3a. Determine what needs to be deleted
		let tickers_to_be_deleted: Vec<i64> = existing_tickers.iter().filter(
			|t| !provided_tickers.contains(t.as_str())
		).filter_map(
			|t| existing_tickers_to_id.get(t).copied()
		).collect();

		// 3b. Delete extras
		for id in tickers_to_be_deleted
		{
			// Delete the security_exchange row dependant on the security_ticker row
			self.t_security_exchange.delete_row(id).await?;

			// Delete the security_ticker row
			self.t_security_ticker.delete_row(id).await?;
		}

		Ok(inserted_ids)
	}
}
