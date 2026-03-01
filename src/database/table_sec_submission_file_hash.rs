use std::sync::Arc;
use sqlx::FromRow;

use super::database_connection::DatabaseConnection;


#[derive(Debug, FromRow)]
pub struct TableSecSubmissionFileHashRow
{
}

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

	pub async fn read_row(
		&self,
		submission_file_name: &str,
		hash: &str,
	) -> Result<Option<TableSecSubmissionFileHashRow>, Box<dyn std::error::Error>>
	{
		let result = sqlx::query_as::<_, TableSecSubmissionFileHashRow>(
			"SELECT * FROM sec_submission_file_hash WHERE submission_file_name = ? AND hash = ?;"
		).bind(
			submission_file_name
		).bind(
			hash
		).fetch_optional(
			self.db_connection.pool()
		).await?;

		Ok(result)
	}
}
