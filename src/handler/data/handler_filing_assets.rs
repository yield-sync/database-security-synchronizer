use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::schema::Assets;

use crate::{ log_debug };
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

	pub async fn synchronize(&self, assets: &Vec<Assets>) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing {} filing_assets..", assets.len());

		self.table_filing_assets.create_rows(assets).await?;

		Ok(())
	}
}
