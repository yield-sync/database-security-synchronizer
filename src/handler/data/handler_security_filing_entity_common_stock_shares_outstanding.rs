use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::schema::CompanyfactsEntityCommonStockSharesOutstanding;

use crate::{ log_debug, log_superdebug };
use crate::database::table_security_filing_entity_common_stock_shares_outstanding::{
	TableSecurityFilingEntityCommonStockSharesOutstanding,
};


pub struct HandlerSecurityFilingEntityCommonStockSharesOutstanding
{
	entity_common_stock_shares_outstanding: TableSecurityFilingEntityCommonStockSharesOutstanding,
}


impl HandlerSecurityFilingEntityCommonStockSharesOutstanding
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			entity_common_stock_shares_outstanding: TableSecurityFilingEntityCommonStockSharesOutstanding::new(
				db_connection.clone()
			),
		}
	}

	pub async fn synchronize(
		&self,
		entity_common_stock_shares_outstanding: &Vec<CompanyfactsEntityCommonStockSharesOutstanding>,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing security_filing_common_stock_shares_outstanding..");

		for csso in entity_common_stock_shares_outstanding
		{
			// Check if the data is already in the database
			if let Some(_) = self.entity_common_stock_shares_outstanding.read_row(
				&csso.security_filing_accession_number
			).await?
			{
				log_superdebug!(
					"Row with security_filing_accession_number {} already exists in database",
					csso.security_filing_accession_number
				);

				continue;
			}

			self.entity_common_stock_shares_outstanding.create_row(&csso).await?;
		}

		Ok(())
	}
}
