use std::sync::Arc;

use super::database_connection::DatabaseConnection;
use crate::schema::CommonStockSharesOutstanding;

use sqlx::FromRow;


#[derive(Debug, FromRow)]
pub struct RowFilingCommonStockSharesOutstanding
{}


pub struct TableFilingCommonStockSharesOutstanding
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableFilingCommonStockSharesOutstanding
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn create_row(
		&self,
		common_stock_shares_outstanding: &CommonStockSharesOutstanding
	) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query(
			r#"
				INSERT INTO filing_common_stock_shares_outstanding (
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
			&common_stock_shares_outstanding.security_filing_accession_number
		).bind(
			&common_stock_shares_outstanding.end
		).bind(
			&common_stock_shares_outstanding.filed
		).bind(
			&common_stock_shares_outstanding.fp
		).bind(
			&common_stock_shares_outstanding.fy
		).bind(
			&common_stock_shares_outstanding.form
		).bind(
			common_stock_shares_outstanding.val
		).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}

	pub async fn read_row(
		&self,
		security_filing_accession_number: &str,
	) -> Result<Option<RowFilingCommonStockSharesOutstanding>, Box<dyn std::error::Error>>
	{
		let existing_row = sqlx::query_as::<_, RowFilingCommonStockSharesOutstanding>(
			"SELECT * FROM filing_common_stock_shares_outstanding WHERE security_filing_accession_number = ?"
		).bind(
			security_filing_accession_number
		).fetch_optional(
			self.db_connection.pool()
		).await?;

		Ok(existing_row)
	}
}
