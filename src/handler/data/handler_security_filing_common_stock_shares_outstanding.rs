use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::database::table_security_filing_common_stock_shares_outstanding::{
	TableSecurityFilingCommonStockSharesOutstanding
};
use crate::schema::CompanyfactsCommonStockSharesOutstanding;


pub struct HandlerSecurityFilingCommonStockSharesOutstanding
{
	t_security_filing_common_stock_shares_outstanding: TableSecurityFilingCommonStockSharesOutstanding,
}


impl HandlerSecurityFilingCommonStockSharesOutstanding
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			t_security_filing_common_stock_shares_outstanding: TableSecurityFilingCommonStockSharesOutstanding::new(
				db_connection.clone()
			),
		}
	}

	pub async fn synchronize(
		&self,
		log_level: u8,
		common_stock_shares_outstanding: &Vec<CompanyfactsCommonStockSharesOutstanding>,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		if log_level >= 1
		{
			println!("\tSynchronizing security common stock shares outstanding..");
		}

		match self.t_security_filing_common_stock_shares_outstanding.create_rows(
			&common_stock_shares_outstanding,
			true
		).await
		{
			Ok(_) => {}

			Err(e) =>
			{
				eprintln!("[WARN] Failed to insert into security_filing_common_stock_shares_outstanding: {}", e);
			}
		}

		Ok(())
	}
}
