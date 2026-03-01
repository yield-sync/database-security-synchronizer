use std::sync::Arc;
use sqlx::Row;

use crate::database::database_connection::DatabaseConnection;
use crate::database::table_security_exchange_ticker::TableSecurityExchangeTicker;

use crate::{ log_debug, log_info, log_superdebug };


pub struct HandlerSecurityExchangeTicker
{
	t_security_exchange_ticker: TableSecurityExchangeTicker,
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
			t_security_exchange_ticker: TableSecurityExchangeTicker::new(db_connection.clone()),
		}
	}

	pub async fn synchronize(
		&self,
		security_cik: &str,
		exchanges: &Vec<String>,
		tickers: &Vec<String>,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!("Synchronizing security_exchange_ticker..");

		if tickers.len() != exchanges.len()
		{
			return Err("Tickers and exchanges must have the same length".into());
		}

		for (i, ticker) in tickers.iter().enumerate()
		{
			let result = self.t_security_exchange_ticker.find_rows(&security_cik, &exchanges[i], &ticker).await?;

			if result.len() == 0
			{
				self.t_security_exchange_ticker.create_row(&security_cik, &exchanges[i], &ticker).await?;
			}
		}

		let existing_rows = self.t_security_exchange_ticker.read_rows(&security_cik).await?;

		let mut row_with_id_to_be_deleted: Vec<i64> = Vec::new();

		for row in existing_rows
		{
			let existing_row_ticker: String = row.get("ticker");
			let existing_row_exchange: String = row.get("exchange");

			log_superdebug!("Verifying row with ticker {} and exchange {}", existing_row_ticker, existing_row_exchange);

			if !tickers.contains(&existing_row_ticker.to_string()) || !exchanges.contains(&existing_row_exchange.to_string())
			{
				row_with_id_to_be_deleted.push(row.get("id"));
			}
		}

		log_debug!("Deleting rows with IDs: {:?}", row_with_id_to_be_deleted);

		for id in row_with_id_to_be_deleted
		{
			self.t_security_exchange_ticker.delete_row(id).await?;
		}

		Ok(())
	}
}
