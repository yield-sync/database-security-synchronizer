use std::sync::Arc;

use sqlx::{ MySql, QueryBuilder };

use super::database_connection::DatabaseConnection;
use crate::schema::EntityCommonStockSharesOutstanding;


pub struct TableFilingEntityCommonStockSharesOutstanding
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableFilingEntityCommonStockSharesOutstanding
{
	// 5 values per row, so 1000 rows stays far below MySQL's 65,535 placeholder limit per query
	const BATCH_SIZE: usize = 1000;


	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	/**
	* Insert rows in batches of BATCH_SIZE, or overwrite the row if a filing already has one. Overwriting lets a
	* re-sync correct rows that were stored with the wrong period before the parser kept only the latest "end" per
	* filing. The parser already dedupes per filing, so a batch never contains the same filing twice.
	*/
	pub async fn upsert_rows(
		&self,
		rows: &[EntityCommonStockSharesOutstanding],
	) -> Result<(), Box<dyn std::error::Error>>
	{
		for batch in rows.chunks(Self::BATCH_SIZE)
		{
			let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
				"INSERT INTO filing_entity_common_stock_shares_outstanding (filing_accession_number, end, fp, fy, val) "
			);

			query_builder.push_values(
				batch,
				|mut row, shares|
				{
					row.push_bind(
						&shares.filing_accession_number
					).push_bind(
						&shares.end
					).push_bind(
						&shares.fp
					).push_bind(
						shares.fy
					).push_bind(
						shares.val
					);
				},
			);

			query_builder.push(
				" ON DUPLICATE KEY UPDATE end = VALUES(end), fp = VALUES(fp), fy = VALUES(fy), val = VALUES(val)"
			);

			query_builder.build().execute(self.db_connection.pool()).await?;
		}

		Ok(())
	}
}
