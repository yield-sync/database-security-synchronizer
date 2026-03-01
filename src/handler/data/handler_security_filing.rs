use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;

use crate::database::table_security_filing::{ TableSecurityFiling, TableSecurityFilingInsertionError };

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
			match self.t_security_filing.create_row(
				&security_cik,
				&f.accession_number,
				&f.form,
				&f.filing_date,
				&f.report_date,
				&f.acceptance,
			).await
			{
				Ok(_) => continue,

				Err(TableSecurityFilingInsertionError::DuplicateEntryError) =>
				{
					log_ultradebug!("WARNING: TableSecurityFiling Duplicate entry error");
				}

				Err(TableSecurityFilingInsertionError::Database(e)) =>
				{
					return Err(e.into());
				}
			}
		}

		Ok(vec![])
	}
}
