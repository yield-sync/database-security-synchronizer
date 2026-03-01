use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::database::table_security::TableSecurity;

use crate::{ log_debug };
use crate::database::table_asset::{ AssetRow, TableAsset };


#[derive(Debug)]
pub struct SynchronizeSecurity
{
	pub cik: String,
	pub business_country: String,
	pub business_city: String,
	pub business_state: String,
	pub business_street1: String,
	pub business_zip: String,
	pub description: String,
	pub ein: String,
	pub entity_type: String,
	pub name: String,
	pub phone: String,
	pub sic: String,
	pub website: String,
}


pub struct HandlerSecurity
{
	t_asset: TableAsset,
	t_security: TableSecurity
}


impl HandlerSecurity
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			t_asset: TableAsset::new(db_connection.clone()),
			t_security: TableSecurity::new(db_connection.clone()),
		}
	}


	/**
	* Ensures a security exists: if not found by CIK, creates asset and security rows.
	* Hash comparison and updates are done by the caller (HandlerSecurityProfile).
	*/
	pub async fn synchronize(
		&self,
		synchronize_security: &SynchronizeSecurity
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing security..");

		if let Some(_) = self.t_security.get_by_cik(&synchronize_security.cik).await?
		{
			log_debug!("Security found in database. Updating it now..");

			self.t_security.update_row(
				&synchronize_security.cik,
				&synchronize_security.business_city,
				&synchronize_security.business_country,
				&synchronize_security.business_state,
				&synchronize_security.business_street1,
				&synchronize_security.business_zip,
				&synchronize_security.description,
				&synchronize_security.ein,
				&synchronize_security.entity_type,
				&synchronize_security.phone,
				&synchronize_security.sic,
				&synchronize_security.website,
			).await?;
		}
		else
		{
			log_debug!("Security not found in database. Inserting it now..");

			// Insert into database
			let asset: AssetRow = self.t_asset.create_row(
				&String::from("Other"),
				&String::from("Other"),
				&synchronize_security.name,
				&String::from("security")
			).await?;

			self.t_security.create_row(
				asset.id,
				&synchronize_security.business_city,
				&synchronize_security.business_country,
				&synchronize_security.business_state,
				&synchronize_security.business_street1,
				&synchronize_security.business_zip,
				&synchronize_security.cik,
				&synchronize_security.description,
				&synchronize_security.ein,
				&synchronize_security.entity_type,
				&synchronize_security.phone,
				&synchronize_security.sic,
				&synchronize_security.website,
			).await?;
		}


		Ok(())
	}
}
