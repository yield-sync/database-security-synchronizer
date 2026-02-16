use std::sync::Arc;
use sqlx::FromRow;
use super::database_connection::DatabaseConnection;


#[derive(Debug, FromRow)]
pub struct AssetRow
{
	pub id: i32,
}

pub struct TableAsset
{
	db_connection: Arc<DatabaseConnection>,
}

impl TableAsset
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn create_row(
		&self,
		industry: &str,
		sector: &str,
		name: &str,
		asset_type: &str
	) -> Result<AssetRow, Box<dyn std::error::Error>>
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
			self.db_connection.pool()
		).await?;

		// Get auto-generated ID
		let id = result.last_insert_id() as i64;

		// Fetch and return the inserted row
		let asset = sqlx::query_as::<_, AssetRow>(
			"SELECT * FROM asset WHERE id = ?"
		).bind(
			id
		).fetch_one(
			self.db_connection.pool()
		).await?;

		Ok(asset)
	}
}
