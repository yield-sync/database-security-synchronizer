use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::schema::Assets;

use crate::{ log_debug, log_superdebug };
use crate::database::table_filing_assets::TableFilingAssets;


pub struct HandlerFilingAssets
{
	table_filing_assets: TableFilingAssets,
}


impl HandlerFilingAssets
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			table_filing_assets: TableFilingAssets::new(db_connection.clone()),
		}
	}

	pub async fn synchronize(&self, assets: &Vec<Assets>,) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing filing_assets..");

		for a in assets
		{
			// Check if the data is already in the database
			if let Some(_) = self.table_filing_assets.read_row(&a.security_filing_accession_number, &a.end).await?
			{
				log_superdebug!(
					"Row with security_filing_accession_number {} already exists in database",
					a.security_filing_accession_number
				);

				continue;
			}

			self.table_filing_assets.create_row(&a).await?;
		}

		Ok(())
	}
}
