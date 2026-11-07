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
			r#"
				INSERT INTO sec_submission_file_hash (submission_file_name, hash)
				VALUES (?, ?)
				ON DUPLICATE KEY UPDATE hash = VALUES(hash)
			"#
		).bind(
			submission_file_name
		).bind(
			hash
		).execute(
			self.db_connection.pool()
		).await?;

		Ok(())
	}

	/**
	* Runs for every file in submissions.zip on every sync, so it only asks whether a matching row exists instead of
	* fetching the row itself
	*/
	pub async fn exists(&self, submission_file_name: &str, hash: &str) -> Result<bool, Box<dyn std::error::Error>>
	{
		let result: Option<(i32,)> = sqlx::query_as(
			"SELECT 1 FROM sec_submission_file_hash WHERE submission_file_name = ? AND hash = ? LIMIT 1"
		).bind(
			submission_file_name
		).bind(
			hash
		).fetch_optional(
			self.db_connection.pool()
		).await?;

		Ok(result.is_some())
	}
}
