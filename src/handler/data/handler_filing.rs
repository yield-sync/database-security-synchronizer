use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;

use crate::database::table_filing::{ TableSecurityFiling, };

use crate::{ log_debug };
use crate::schema::{ SubmissionsDataFilings };


pub struct HandlerFiling
{
	t_filing: TableSecurityFiling,
}


impl HandlerFiling
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			t_filing: TableSecurityFiling::new(db_connection.clone()),
		}
	}

	pub async fn synchronize(
		&self,
		security_cik: &str,
		filings: &Vec<SubmissionsDataFilings>,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing {} filings..", filings.len());

		self.t_filing.create_rows(security_cik, filings).await?;

		Ok(())
	}
}
