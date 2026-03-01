use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;

use crate::database::table_sec_submission_file_hash::{ TableSecSubmissionFileHash };

use crate::{ log_debug, log_superdebug };


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

	pub async fn synchronize(
		&self,
		submission_file_name: &str,
		sec_submission_file_hash: &str,
	) -> Result<Vec<i64>, Box<dyn std::error::Error>>
	{
		log_debug!("Synchronizing table_sec_submission_file_hash..");

		log_superdebug!("s_file_name: {} s_hash: {}", submission_file_name, sec_submission_file_hash);

		self.table_sec_submission_file_hash.create_row(&submission_file_name, &sec_submission_file_hash).await?;

		Ok(vec![])
	}

	pub async fn hash_exists(
		&self,
		submission_file_name: &str,
		sec_submission_file_hash: &str,
	) -> Result<bool, Box<dyn std::error::Error>>
	{
		let result = self.table_sec_submission_file_hash.read_row(
			&submission_file_name,
			&sec_submission_file_hash,
		).await?;

		if let Some(_) = result
		{
			Ok(true)
		}
		else
		{
			Ok(false)
		}
	}
}
