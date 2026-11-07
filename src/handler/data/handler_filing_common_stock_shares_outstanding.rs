use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::schema::CommonStockSharesOutstanding;

use crate::{ log_debug };
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
		log_debug!(
			"Synchronizing {} filing_common_stock_shares_outstanding..",
			common_stock_shares_outstanding.len(),
		);

		self.table_filing_common_stock_shares_outstanding.upsert_rows(
			common_stock_shares_outstanding
		).await?;

		Ok(())
	}
}
