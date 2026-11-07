use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::schema::EntityCommonStockSharesOutstanding;

use crate::{ log_debug };
use crate::database::table_filing_entity_common_stock_shares_outstanding::TableFilingEntityCommonStockSharesOutstanding;


pub struct HandlerFilingEntityCommonStockSharesOutstanding
{
	table_filing_entity_common_stock_shares_outstanding: TableFilingEntityCommonStockSharesOutstanding,
}


impl HandlerFilingEntityCommonStockSharesOutstanding
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			table_filing_entity_common_stock_shares_outstanding: TableFilingEntityCommonStockSharesOutstanding::new(
				db_connection.clone()
			),
		}
	}

	pub async fn synchronize(
		&self,
		entity_common_stock_shares_outstanding: &Vec<EntityCommonStockSharesOutstanding>,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing filing_entity_common_stock_shares_outstanding..");

		for ecsso in entity_common_stock_shares_outstanding
		{
			self.table_filing_entity_common_stock_shares_outstanding.upsert_row(&ecsso).await?;
		}

		Ok(())
	}
}
