use std::sync::Arc;

use sqlx::{ MySql, QueryBuilder };

use super::database_connection::DatabaseConnection;
use crate::schema::SubmissionsDataFilings;


pub struct TableSecurityFiling
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableSecurityFiling
{
	// 6 values per row, so 1000 rows stays far below MySQL's 65,535 placeholder limit per query
	const BATCH_SIZE: usize = 1000;


	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	/**
	* Insert filings in batches of BATCH_SIZE rows per query. Filings that already exist are left untouched.
	*
	* "ON DUPLICATE KEY UPDATE accession_number = accession_number" is a no-op update: it only swallows duplicate key
	* conflicts, so every other error (bad value, missing security, ...) still fails loudly. INSERT IGNORE would turn
	* those into silent warnings.
	*/
	pub async fn create_rows(
		&self,
		security_cik: &str,
		filings: &[SubmissionsDataFilings],
	) -> Result<(), Box<dyn std::error::Error>>
	{
		for batch in filings.chunks(Self::BATCH_SIZE)
		{
			let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
				"INSERT INTO filing (security_cik, accession_number, form, filing_date, report_date, acceptance) "
			);

			query_builder.push_values(
				batch,
				|mut row, filing|
				{
					row.push_bind(
						security_cik
					).push_bind(
						&filing.accession_number
					).push_bind(
						&filing.form
					).push_bind(
						filing.filing_date
					).push_bind(
						filing.report_date
					).push_bind(
						filing.acceptance
					);
				},
			);

			query_builder.push(" ON DUPLICATE KEY UPDATE accession_number = accession_number");

			query_builder.build().execute(self.db_connection.pool()).await?;
		}

		Ok(())
	}
}
