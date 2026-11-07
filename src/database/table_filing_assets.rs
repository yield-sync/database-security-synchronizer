use std::sync::Arc;

use sqlx::{ MySql, QueryBuilder };

use super::database_connection::DatabaseConnection;
use crate::schema::Assets;


pub struct TableFilingAssets
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableFilingAssets
{
	// 5 values per row, so 1000 rows stays far below MySQL's 65,535 placeholder limit per query
	const BATCH_SIZE: usize = 1000;


	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	/**
	* Insert assets in batches of BATCH_SIZE rows per query. Rows that already exist for the same
	* (filing_accession_number, end) are left untouched, which also covers duplicates inside the same batch.
	*
	* The no-op "ON DUPLICATE KEY UPDATE" only swallows duplicate key conflicts, so every other error still fails.
	*/
	pub async fn create_rows(&self, assets: &[Assets]) -> Result<(), Box<dyn std::error::Error>>
	{
		for batch in assets.chunks(Self::BATCH_SIZE)
		{
			let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
				"INSERT INTO filing_assets (filing_accession_number, end, fp, fy, val) "
			);

			query_builder.push_values(
				batch,
				|mut row, asset|
				{
					row.push_bind(
						&asset.filing_accession_number
					).push_bind(
						&asset.end
					).push_bind(
						&asset.fp
					).push_bind(
						asset.fy
					).push_bind(
						asset.val
					);
				},
			);

			query_builder.push(" ON DUPLICATE KEY UPDATE filing_accession_number = filing_accession_number");

			query_builder.build().execute(self.db_connection.pool()).await?;
		}

		Ok(())
	}
}
