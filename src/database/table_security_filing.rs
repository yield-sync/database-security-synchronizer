use std::sync::Arc;

use chrono::{NaiveDate, NaiveDateTime};

use super::database_connection::DatabaseConnection;


pub enum TableSecurityFilingInsertionError
{
	DuplicateEntryError,
	Database(sqlx::Error),
}

pub struct TableSecurityFiling
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableSecurityFiling
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn create_row(
		&self,
		security_cik: &str,
		accession_number: &str,
		form: &str,
		filing_date: &NaiveDate,
		report_date: &Option<NaiveDate>,
		acceptance: &NaiveDateTime,
	) -> Result<(), TableSecurityFilingInsertionError>
	{
		match sqlx::query(
			r#"
				INSERT INTO security_filing (security_cik, accession_number, form, filing_date, report_date, acceptance)
				VALUES (?, ?, ?, ?, ?, ?)
			"#,
		).bind(
			security_cik
		).bind(
			accession_number
		).bind(
			form
		).bind(
			filing_date
		).bind(
			report_date
		).bind(
			acceptance
		).execute(
			self.db_connection.pool()
		).await
		{
			Ok(_) => Ok(()),

			Err(e) =>
			{
				// Catch duplicate entry error
				if e.to_string().contains("error returned from database: 1062")
				{
					return Err(TableSecurityFilingInsertionError::DuplicateEntryError);
				}

			 	return Err(TableSecurityFilingInsertionError::Database(e));
			}
		}
	}
}
