use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::schema::CommonStockSharesOutstanding;

use crate::{ log_debug, log_superdebug, };
use crate::database::table_filing_common_stock_shares_outstanding::TableFilingCommonStockSharesOutstanding;


pub struct HandlerFilingCommonStockSharesOutstanding
{
	table_filing_common_stock_shares_outstanding: TableFilingCommonStockSharesOutstanding,
}


impl HandlerFilingCommonStockSharesOutstanding
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			table_filing_common_stock_shares_outstanding: TableFilingCommonStockSharesOutstanding::new(
				db_connection.clone()
			),
		}
	}

	pub async fn synchronize(
		&self,
		common_stock_shares_outstanding: &Vec<CommonStockSharesOutstanding>,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing filing_common_stock_shares_outstanding..");

		for csso in common_stock_shares_outstanding
		{
			// Check if the data is already in the database
			if let Some(_) = self.table_filing_common_stock_shares_outstanding.read_row(
				&csso.security_filing_accession_number
			).await?
			{
				log_superdebug!(
					"Row with security_filing_accession_number {} already exists in database",
					csso.security_filing_accession_number
				);

				continue;
			}

			self.table_filing_common_stock_shares_outstanding.create_row(&csso).await?;
		}

		Ok(())
	}
}
