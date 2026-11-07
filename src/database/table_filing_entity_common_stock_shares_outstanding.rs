use std::sync::Arc;

use super::database_connection::DatabaseConnection;
use crate::schema::EntityCommonStockSharesOutstanding;


pub struct TableFilingEntityCommonStockSharesOutstanding
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableFilingEntityCommonStockSharesOutstanding
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	/**
	* Insert the row, or overwrite it if this filing already has one. Overwriting lets a re-sync correct rows that
	* were stored with the wrong period before the parser kept only the latest "end" per filing.
	*/
	pub async fn upsert_row(
		&self,
		entity_common_stock_shares_outstanding: &EntityCommonStockSharesOutstanding,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query(
			r#"
				INSERT INTO filing_entity_common_stock_shares_outstanding (
					filing_accession_number,
					end,
					fp,
					fy,
					val
				)
				VALUES (?, ?, ?, ?, ?)
				ON DUPLICATE KEY UPDATE
					end = VALUES(end),
					fp = VALUES(fp),
					fy = VALUES(fy),
					val = VALUES(val)
			"#
		).bind(
			&entity_common_stock_shares_outstanding.filing_accession_number
		).bind(
			&entity_common_stock_shares_outstanding.end
		).bind(
			&entity_common_stock_shares_outstanding.fp
		).bind(
			&entity_common_stock_shares_outstanding.fy
		).bind(
			entity_common_stock_shares_outstanding.val
		).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}
}
