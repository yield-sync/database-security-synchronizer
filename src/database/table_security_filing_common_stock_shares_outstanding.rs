use std::sync::Arc;

use super::database_connection::DatabaseConnection;
use crate::schema::CompanyfactsCommonStockSharesOutstanding;

use crate::{ log_info, log_warn };


pub enum TableSecurityFilingCommonStockSharesOutstandingInsertionError
{
	ForeignKeyNotFoundError,
	Database(sqlx::Error),
}

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
		security_filing_accession_number: &str,
		end: &str,
		filed: &str,
		fp: &str,
		fy: &i64,
		form: &str,
		common_stock_shares_outstanding: &i64,
	) -> Result<(), TableSecurityFilingCommonStockSharesOutstandingInsertionError>
	{
		match sqlx::query(
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
		).bind(security_filing_accession_number).bind(end).bind(filed).bind(fp).bind(fy).bind(form).bind(
			common_stock_shares_outstanding
		).execute(
			self.db_connection.pool()
		).await
		{
			Ok(_) => Ok(()),

			Err(e) =>
			{
				// Foreign key not found error
				if e.to_string().contains("error returned from database: 1452")
				{
					return Err(TableSecurityFilingCommonStockSharesOutstandingInsertionError::ForeignKeyNotFoundError);
				}

			 	return Err(TableSecurityFilingCommonStockSharesOutstandingInsertionError::Database(e));
			}
		}
	}

	pub async fn create_rows(
		&self,
		companyfacts_common_stock_shares_outstanding: &Vec<CompanyfactsCommonStockSharesOutstanding>,
		ignore_foreign_key_not_found_error: bool,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!(
			"Inserting into security_filing_common_stock_shares_outstanding. projected row count: {}",
			companyfacts_common_stock_shares_outstanding.len()
		);

		for ccsso in companyfacts_common_stock_shares_outstanding
		{
			match self.create_row(
				&ccsso.security_filing_accession_number,
				&ccsso.end,
				&ccsso.filed,
				&ccsso.fp,
				&ccsso.fy,
				&ccsso.form,
				&ccsso.common_stock_shares_outstanding,
			).await
			{
				Ok(_) => continue,

				Err(TableSecurityFilingCommonStockSharesOutstandingInsertionError::ForeignKeyNotFoundError) =>
				{
					let error_message = "Foreign key not found";

					if ignore_foreign_key_not_found_error
					{
						log_warn!("[WARN] {}, Skipping..", error_message);

						continue;
					}

					return Err(error_message.into());
				}

				Err(TableSecurityFilingCommonStockSharesOutstandingInsertionError::Database(e)) =>
				{
					return Err(Box::new(e));
				}
			}
		}

		Ok(())
	}
}
