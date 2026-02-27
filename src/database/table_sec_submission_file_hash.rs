use std::sync::Arc;
use super::database_connection::DatabaseConnection;


pub struct TableSecSubmissionFileHash
{
	db_connection: Arc<DatabaseConnection>,
}

impl TableSecSubmissionFileHash
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn create_row(&self, submission_file_name: &str, hash: &str,) -> Result<(), Box<dyn std::error::Error>>
	{
		sqlx::query(
			"INSERT INTO sec_submission_file_hash (submission_file_name, hash) VALUES (?, ?) ON DUPLICATE KEY UPDATE hash = VALUES(hash);"
		).bind(
			submission_file_name
		).bind(
			hash
		).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}
}
