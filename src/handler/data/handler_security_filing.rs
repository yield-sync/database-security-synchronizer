use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;

use crate::database::table_security_filing::{TableSecurityFiling, TableSecurityFilingInsertionError};
use crate::schema::{SubmissionsDataFilings};


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
		log_level: u8,
		security_cik: &str,
		filings: &Vec<SubmissionsDataFilings>,
	) -> Result<Vec<i64>, Box<dyn std::error::Error>>
	{
		if log_level >= 1
		{
			println!("\tSynchronizing security filings..");
		}
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
					eprintln!(
						"\t[WARN] TableSecurityFiling Duplicate entry error"
					);
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
