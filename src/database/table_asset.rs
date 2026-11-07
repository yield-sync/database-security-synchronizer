use std::sync::Arc;

use sqlx::{ Executor, MySql };

use super::database_connection::DatabaseConnection;


pub struct TableAsset
{
	#[allow(dead_code)]
	db_connection: Arc<DatabaseConnection>,
}


impl TableAsset
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	/**
	* Insert an asset and return its auto-generated id.
	*
	* Accepts any executor (the pool, a connection, or a transaction) so the caller decides whether this insert is
	* part of a transaction.
	*/
	pub async fn create_row<'executor, TExecutor>(
		&self,
		executor: TExecutor,
		industry: &str,
		sector: &str,
		name: &str,
		asset_type: &str,
	) -> Result<i32, Box<dyn std::error::Error>>
	where
		TExecutor: Executor<'executor, Database = MySql>,
	{
		let result = sqlx::query("INSERT INTO asset (industry, sector, name, `type`) VALUES (?, ?, ?, ?)").bind(
			industry
		).bind(
			sector
		).bind(
			name
		).bind(
			asset_type
		).execute(
			executor
		).await?;

		let id: i32 = i32::try_from(result.last_insert_id())?;

		Ok(id)
	}

	/**
	* Update the name of the asset that belongs to the security with this CIK
	*/
	pub async fn update_name_by_security_cik<'executor, TExecutor>(
		&self,
		executor: TExecutor,
		security_cik: &str,
		name: &str,
	) -> Result<(), Box<dyn std::error::Error>>
	where
		TExecutor: Executor<'executor, Database = MySql>,
	{
		sqlx::query(
			r#"
				UPDATE asset
				JOIN security ON security.asset_id = asset.id
				SET asset.name = ?
				WHERE security.cik = ?
			"#
		).bind(
			name
		).bind(
			security_cik
		).execute(
			executor
		).await?;

		Ok(())
	}
}
