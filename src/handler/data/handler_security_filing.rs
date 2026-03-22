use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;

use crate::database::table_security_filing::{ TableSecurityFiling, };

use crate::{ log_debug, log_ultradebug };
use crate::schema::{ SubmissionsDataFilings };


pub struct HandlerSecurityFiling
{
	t_security_filing: TableSecurityFiling,
}


impl HandlerSecurityFiling
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			t_security_filing: TableSecurityFiling::new(db_connection.clone()),
		}
	}

	pub async fn synchronize(
	&self,
		security_cik: &str,
		filings: &Vec<SubmissionsDataFilings>,
	) -> Result<Vec<i64>, Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing security_filings..");

		for f in filings
		{
			if let Some(_) = self.t_security_filing.read_row(&f.accession_number).await?
			{
				log_ultradebug!(
					"Row with accession_number {} already exists in database",
					f.accession_number
				);

				continue;
			}

			self.t_security_filing.create_row(
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
