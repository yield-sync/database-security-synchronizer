use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::schema::CompanyfactsEntityCommonStockSharesOutstanding;

use crate::{ log_debug, log_warn };
use crate::database::table_security_filing_entity_common_stock_shares_outstanding::{
	TableSecurityFilingEntityCommonStockSharesOutstanding
};


pub struct HandlerSecurityFilingEntityCommonStockSharesOutstanding
{
	t_s_f_entity_common_stock_shares_outstanding: TableSecurityFilingEntityCommonStockSharesOutstanding,
}


impl HandlerSecurityFilingEntityCommonStockSharesOutstanding
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			t_s_f_entity_common_stock_shares_outstanding: TableSecurityFilingEntityCommonStockSharesOutstanding::new(
				db_connection.clone()
			),
		}
	}

	pub async fn synchronize(
		&self,
		entity_common_stock_shares_outstanding: &Vec<CompanyfactsEntityCommonStockSharesOutstanding>,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing security_filing_common_stock_shares_outstanding..");

		match self.t_s_f_entity_common_stock_shares_outstanding.create_rows(
			&entity_common_stock_shares_outstanding,
			true
		).await
		{
			Ok(_) => {}

			Err(e) =>
			{
				log_warn!("Failed to insert into security_filing_entity_common_stock_shares_outstanding: {}", e);
			}
		}

		Ok(())
	}
}
