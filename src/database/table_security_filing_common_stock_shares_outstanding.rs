use std::sync::Arc;

use super::database_connection::DatabaseConnection;
use crate::schema::CompanyfactsCommonStockSharesOutstanding;

use sqlx::FromRow;


#[derive(Debug, FromRow)]
pub struct TableSecurityFilingCommonStockSharesOutstandingRow
{}


pub struct TableSecurityFilingCommonStockSharesOutstanding
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableSecurityFilingCommonStockSharesOutstanding
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn create_row(
		&self,
		companyfacts_common_stock_shares_outstanding: &CompanyfactsCommonStockSharesOutstanding
	) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query(
			r#"
				INSERT INTO security_filing_common_stock_shares_outstanding (
					security_filing_accession_number,
					end,
					filed,
					fp,
					fy,
					form,
					common_stock_shares_outstanding
				)
				VALUES (?, ?, ?, ?, ?, ?, ?)
			"#
		).bind(&companyfacts_common_stock_shares_outstanding.security_filing_accession_number).bind(
			&companyfacts_common_stock_shares_outstanding.end
		).bind(
			&companyfacts_common_stock_shares_outstanding.filed
		).bind(
			&companyfacts_common_stock_shares_outstanding.fp
		).bind(
			&companyfacts_common_stock_shares_outstanding.fy
		).bind(
			&companyfacts_common_stock_shares_outstanding.form
		).bind(
			companyfacts_common_stock_shares_outstanding.val
		).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}

	pub async fn read_row(
		&self,
		security_filing_accession_number: &str,
	) -> Result<Option<TableSecurityFilingCommonStockSharesOutstandingRow>, Box<dyn std::error::Error>>
	{
		let existing_row = sqlx::query_as::<_, TableSecurityFilingCommonStockSharesOutstandingRow>(
			"SELECT * FROM security_filing_common_stock_shares_outstanding WHERE security_filing_accession_number = ?"
		).bind(
			security_filing_accession_number
		).fetch_optional(
			self.db_connection.pool()
		).await?;

		Ok(existing_row)
	}
}
