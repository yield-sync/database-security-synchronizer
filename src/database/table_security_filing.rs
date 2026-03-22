use std::sync::Arc;

use chrono::{ NaiveDate, NaiveDateTime };

use super::database_connection::DatabaseConnection;

use sqlx::FromRow;


#[derive(Debug, FromRow)]
pub struct RowSecurityFiling
{}


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
	) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query(
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
		).await?;

		Ok(())
	}

	pub async fn read_row(
			&self,
			accession_number: &str,
		) -> Result<Option<RowSecurityFiling>, Box<dyn std::error::Error>>
		{
			let existing_row = sqlx::query_as::<_, RowSecurityFiling>(
				"SELECT * FROM security_filing WHERE accession_number = ?"
			).bind(
				accession_number
			).fetch_optional(
				self.db_connection.pool()
			).await?;

			Ok(existing_row)
		}
}
