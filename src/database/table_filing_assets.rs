use std::sync::Arc;

use super::database_connection::DatabaseConnection;
use crate::schema::Assets;

use sqlx::FromRow;


#[derive(Debug, FromRow)]
pub struct RowFilingAssets
{}


pub struct TableFilingAssets
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableFilingAssets
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn create_row(&self, assets: &Assets) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query(
			r#"
				INSERT INTO filing_assets (
					security_filing_accession_number,
					end,
					filed,
					fp,
					fy,
					form,
					val
				)
				VALUES (?, ?, ?, ?, ?, ?, ?)
			"#
		).bind(
			&assets.security_filing_accession_number
		).bind(
			&assets.end
		).bind(
			&assets.filed
		).bind(
			&assets.fp
		).bind(
			&assets.fy
		).bind(
			&assets.form
		).bind(
			assets.val
		).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}

	pub async fn read_row(
		&self,
		security_filing_accession_number: &str,
	) -> Result<Option<RowFilingAssets>, Box<dyn std::error::Error>>
	{
		let existing_row = sqlx::query_as::<_, RowFilingAssets>(
			"SELECT * FROM filing_assets WHERE security_filing_accession_number = ?"
		).bind(
			security_filing_accession_number
		).fetch_optional(
			self.db_connection.pool()
		).await?;

		Ok(existing_row)
	}
}
