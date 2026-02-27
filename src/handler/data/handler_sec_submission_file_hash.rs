use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;

use crate::database::table_sec_submission_file_hash::{ TableSecSubmissionFileHash };

use crate::{ log_info, log_debug };


pub struct HandlerSecSubmissionFileHash
{
	table_sec_submission_file_hash: TableSecSubmissionFileHash,
}


impl HandlerSecSubmissionFileHash
{
	/**
	* @visibility: Public
	*/
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self
		{
			table_sec_submission_file_hash: TableSecSubmissionFileHash::new(db_connection.clone()),
		}
	}

	pub async fn synchronize(&self, s_file_name: &str, s_hash: &str,) -> Result<Vec<i64>, Box<dyn std::error::Error>>
	{
		log_info!("Synchronizing table_sec_submission_file_hash {} {}..", s_file_name, s_hash);

		self.table_sec_submission_file_hash.create_row(&s_file_name, &s_hash).await?;

		Ok(vec![])
	}
}
