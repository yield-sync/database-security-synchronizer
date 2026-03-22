use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::schema::CompanyfactsEntityCommonStockSharesOutstanding;

use crate::{ log_debug, log_superdebug };
use crate::database::table_security_filing_entity_common_stock_shares_outstanding::{
	TableSecurityFilingEntityCommonStockSharesOutstanding,
	TableSecurityFilingEntityCommonStockSharesOutstandingInsertError,
};


pub struct HandlerSecurityFilingEntityCommonStockSharesOutstanding
{
	t_s_f_entity_common_stock_shares_outstanding: TableSecurityFilingEntityCommonStockSharesOutstanding,
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
			t_s_f_entity_common_stock_shares_outstanding: TableSecurityFilingEntityCommonStockSharesOutstanding::new(
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
			if let Some(_) = self.t_s_f_entity_common_stock_shares_outstanding.read_row(
				&csso.security_filing_accession_number
			).await?
			{
				log_superdebug!(
					"Row with security_filing_accession_number {} already exists in database",
					csso.security_filing_accession_number
				);
			}
			else
			{
				match self.t_s_f_entity_common_stock_shares_outstanding.create_row(&csso).await
				{
					Ok(_) => {}

					Err(TableSecurityFilingEntityCommonStockSharesOutstandingInsertError::ForeignKeyNotFoundError) =>
					{
						let error_message = format!(
							"security_filing_accession_number (Foreign key) not found Error: {}",
							csso.security_filing_accession_number
						);

						return Err(error_message.into());
					}

					Err(TableSecurityFilingEntityCommonStockSharesOutstandingInsertError::Uncaught(e)) =>
					{
						return Err(e.into());
					}
				}
			}
		}

		Ok(())
	}
}
