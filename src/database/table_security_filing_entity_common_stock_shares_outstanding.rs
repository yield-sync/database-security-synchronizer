use std::sync::Arc;

use super::database_connection::DatabaseConnection;
use crate::schema::CompanyfactsEntityCommonStockSharesOutstanding;

use crate::{ log_info, log_warn };


pub enum TableSecurityFilingEntityCommonStockSharesOutstandingInsertionError
{
	ForeignKeyNotFoundError,
	Database(sqlx::Error),
}

pub struct TableSecurityFilingEntityCommonStockSharesOutstanding
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableSecurityFilingEntityCommonStockSharesOutstanding
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn create_row(
		&self,
		companyfacts_entity_common_stock_shares_outstanding: &CompanyfactsEntityCommonStockSharesOutstanding,
	) -> Result<(), TableSecurityFilingEntityCommonStockSharesOutstandingInsertionError>
	{
		match sqlx::query(
			r#"
				INSERT INTO security_filing_entity_common_stock_shares_outstanding (
					security_filing_accession_number,
					end,
					filed,
					fp,
					fy,
					form,
					entity_common_stock_shares_outstanding
				)
				VALUES (?, ?, ?, ?, ?, ?, ?)
			"#
		).bind(&companyfacts_entity_common_stock_shares_outstanding.security_filing_accession_number).bind(
			&companyfacts_entity_common_stock_shares_outstanding.end
		).bind(
			&companyfacts_entity_common_stock_shares_outstanding.filed
		).bind(
			&companyfacts_entity_common_stock_shares_outstanding.fp
		).bind(
			&companyfacts_entity_common_stock_shares_outstanding.fy
		).bind(
			&companyfacts_entity_common_stock_shares_outstanding.form
		).bind(
			&companyfacts_entity_common_stock_shares_outstanding.entity_common_stock_shares_outstanding
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
					return Err(
						TableSecurityFilingEntityCommonStockSharesOutstandingInsertionError::ForeignKeyNotFoundError
					);
				}

			 	return Err(TableSecurityFilingEntityCommonStockSharesOutstandingInsertionError::Database(e));
			}
		}
	}

	pub async fn create_rows(
		&self,
		companyfacts_entity_common_stock_shares_outstandings: &Vec<CompanyfactsEntityCommonStockSharesOutstanding>,
		ignore_foreign_key_not_found_error: bool,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!(
			"Inserting into security_filing_entity_common_stock_shares_outstanding. projected row count: {}",
			companyfacts_entity_common_stock_shares_outstandings.len()
		);

		for cecsso in companyfacts_entity_common_stock_shares_outstandings
		{
			match self.create_row(&cecsso).await
			{
				Ok(_) => continue,

				Err(TableSecurityFilingEntityCommonStockSharesOutstandingInsertionError::ForeignKeyNotFoundError) =>
				{
					let error_message = "Foreign key not found";

					if ignore_foreign_key_not_found_error
					{
						log_warn!(
							"[TableSecurityFilingEntityCommonStockSharesOutstandingInsertionError] {}, Skipping..",
							error_message
						);

						continue;
					}

					return Err(error_message.into());
				}

				Err(TableSecurityFilingEntityCommonStockSharesOutstandingInsertionError::Database(e)) =>
				{
					return Err(Box::new(e));
				}
			}
		}

		Ok(())
	}
}
