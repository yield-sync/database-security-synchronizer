use std::collections::HashSet;
use std::sync::Arc;
use sqlx::Row;

use crate::database::database_connection::DatabaseConnection;
use crate::database::table_security_exchange_ticker::TableSecurityExchangeTicker;

use crate::{ log_debug, log_superdebug };


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
		exchanges: &[String],
		tickers: &[String],
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing security_exchange_ticker..");

		if tickers.len() != exchanges.len()
		{
			return Err("Tickers and exchanges must have the same length".into());
		}

		// Desired state: (ticker, exchange) pairs from the submission
		let desired: HashSet<(&str, &str)> = tickers.iter().zip(exchanges.iter()).map(
			|(ticker, exchange)| (ticker.as_str(), exchange.as_str())
		).collect();

		// Current state: read once
		let existing_rows = self.t_security_exchange_ticker.read_rows(security_cik).await?;

		let mut existing: HashSet<(String, String)> = HashSet::new();
		let mut stale: HashSet<(String, String)> = HashSet::new();

		for row in existing_rows
		{
			let ticker: String = row.get("ticker");
			let exchange: String = row.get("exchange");

			log_superdebug!("Verifying row with ticker {} and exchange {}", ticker, exchange);

			if desired.contains(&(ticker.as_str(), exchange.as_str()))
			{
				existing.insert((ticker, exchange));
			}
			else
			{
				stale.insert((ticker, exchange));
			}
		}

		log_debug!("Deleting stale (ticker, exchange) pairs: {:?}", stale);

		for (ticker, exchange) in stale
		{
			self.t_security_exchange_ticker.delete_row(exchange, ticker).await?;
		}

		// Insert pairs that don't exist yet
		for (ticker, exchange) in desired
		{
			if !existing.contains(&(ticker.to_string(), exchange.to_string()))
			{
				log_superdebug!("Creating row with ticker {} and exchange {}", ticker, exchange);

				self.t_security_exchange_ticker.create_row(security_cik, exchange, ticker).await?;
			}
		}

		Ok(())
	}
}
