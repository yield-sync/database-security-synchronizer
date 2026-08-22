use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;

use crate::database::table_filing::{ TableSecurityFiling, };

use crate::{ log_debug, log_ultradebug };
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
	) -> Result<Vec<i64>, Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing filings..");

		for f in filings
		{
			if let Some(_) = self.t_filing.read_row(&f.accession_number).await?
			{
				log_ultradebug!(
					"Row with accession_number {} already exists in database",
					f.accession_number
				);

				continue;
			}

			self.t_filing.create_row(
				&security_cik,
				&f.accession_number,
				&f.form,
				&f.filing_date,
				&f.report_date,
				&f.acceptance,
			).await?;
		}

		Ok(vec![])
	}
}
