use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::schema::CompanyfactsCommonStockSharesOutstanding;

use crate::{ log_debug, log_superdebug, };
use crate::database::table_security_filing_common_stock_shares_outstanding::{
	TableSecurityFilingCommonStockSharesOutstanding,
	TableSecurityFilingCommonStockSharesOutstandingInsertError,
};


pub struct HandlerSecurityFilingCommonStockSharesOutstanding
{
	t_security_filing_common_stock_shares_outstanding: TableSecurityFilingCommonStockSharesOutstanding,
}


impl HandlerSecurityFilingCommonStockSharesOutstanding
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			t_security_filing_common_stock_shares_outstanding: TableSecurityFilingCommonStockSharesOutstanding::new(
				db_connection.clone()
			),
		}
	}

	pub async fn synchronize(
		&self,
		common_stock_shares_outstanding: &Vec<CompanyfactsCommonStockSharesOutstanding>,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing security_filing_common_stock_shares_outstanding..");

		for csso in common_stock_shares_outstanding
		{
			// Check if the data is already in the database
			if let Some(_) = self.t_security_filing_common_stock_shares_outstanding.read_row(
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
				match self.t_security_filing_common_stock_shares_outstanding.create_row(&csso).await
				{
					Ok(_) => {}

					Err(TableSecurityFilingCommonStockSharesOutstandingInsertError::ForeignKeyNotFoundError) =>
					{
						let error_message = format!(
							"security_filing_accession_number (Foreign key) not found Error: {}",
							csso.security_filing_accession_number
						);

						return Err(error_message.into());
					}

					Err(TableSecurityFilingCommonStockSharesOutstandingInsertError::Uncaught(e)) =>
					{
						return Err(e.into());
					}
				}
			}
		}

		Ok(())
	}
}
