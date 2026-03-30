use std::sync::Arc;

use super::database_connection::DatabaseConnection;
use crate::schema::EntityCommonStockSharesOutstanding;

use sqlx::FromRow;


#[derive(Debug, FromRow)]
pub struct RowFilingEntityCommonStockSharesOutstanding
{}


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

	pub async fn create_row(
		&self,
		entity_common_stock_shares_outstanding: &EntityCommonStockSharesOutstanding,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query(
			r#"
				INSERT INTO filing_entity_common_stock_shares_outstanding (
					security_filing_accession_number,
					end,
					fp,
					fy,
					val
				)
				VALUES (?, ?, ?, ?, ?)
			"#
		).bind(
			&entity_common_stock_shares_outstanding.security_filing_accession_number
		).bind(
			&entity_common_stock_shares_outstanding.end
		).bind(
			&entity_common_stock_shares_outstanding.fp
		).bind(
			&entity_common_stock_shares_outstanding.fy
		).bind(
			&entity_common_stock_shares_outstanding.val
		).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}

	pub async fn read_row(
		&self,
		security_filing_accession_number: &str,
	) -> Result<Option<RowFilingEntityCommonStockSharesOutstanding>, Box<dyn std::error::Error>>
	{
		let existing_row = sqlx::query_as::<_, RowFilingEntityCommonStockSharesOutstanding>(
			"SELECT * FROM filing_entity_common_stock_shares_outstanding WHERE security_filing_accession_number = ?"
		).bind(
			security_filing_accession_number
		).fetch_optional(
			self.db_connection.pool()
		).await?;

		Ok(existing_row)
	}
}
