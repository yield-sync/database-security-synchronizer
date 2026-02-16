use std::sync::Arc;
use sqlx::FromRow;
use super::database_connection::DatabaseConnection;


#[derive(Debug, FromRow)]
pub struct SecurityRow
{
}

pub struct TableSecurity
{
	db_connection: Arc<DatabaseConnection>,
}


impl TableSecurity
{
	pub fn new(db_connection: Arc<DatabaseConnection>) -> Self
	{
		Self { db_connection }
	}

	pub async fn get_by_cik(&self, cik: &str) -> Result<Option<SecurityRow>, Box<dyn std::error::Error>>
	{
		let row = sqlx::query_as::<_, SecurityRow>("SELECT cik FROM security WHERE cik = ?").bind(cik).fetch_optional(
			&*self.db_connection.pool()
		).await?;

		Ok(row)
	}

	pub async fn create_row(
		&self,
		asset_id: i32,
		business_city: &str,
		business_country: &str,
		business_state: &str,
		business_street1: &str,
		business_zip: &str,
		cik: &str,
		description: &str,
		ein: &str,
		entity_type: &str,
		phone: &str,
		sic: &str,
		website: &str,
	) -> Result<SecurityRow, Box<dyn std::error::Error>>
	{
		sqlx::query(
			r#"
				INSERT INTO security (
					asset_id,
					business_street1,
					business_city,
					business_country,
					business_state,
					business_zip,
					cik,
					description,
					ein,
					entity_type,
					phone,
					sic,
					website
				) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
			"#
		).bind(
			asset_id
		).bind(
			business_street1
		).bind(
			business_city
		).bind(
			business_country
		).bind(
			business_state
		).bind(
			business_zip
		).bind(
			cik
		).bind(
			description
		).bind(
			ein
		).bind(
			entity_type
		).bind(
			phone
		).bind(
			sic
		).bind(
			website
		).execute(
			self.db_connection.pool()
		).await?;

		// Fetch and return the inserted row
		let security = sqlx::query_as::<_, SecurityRow>(
			"SELECT * FROM security WHERE cik = ?"
		).bind(
	   		cik
		).fetch_one(
			self.db_connection.pool()
		).await?;

		Ok(security)
	}

	pub async fn update_row(
		&self,
		cik: &str,
		business_city: &str,
		business_country: &str,
		business_state: &str,
		business_street1: &str,
		business_zip: &str,
		description: &str,
		ein: &str,
		entity_type: &str,
		phone: &str,
		sic: &str,
		website: &str,
	) -> Result<SecurityRow, Box<dyn std::error::Error>>
	{
		sqlx::query(
			r#"
				UPDATE security
				SET
					business_city = ?,
					business_country = ?,
					business_state = ?,
					business_street1 = ?,
					business_zip = ?,
					description = ?,
					ein = ?,
					entity_type = ?,
					phone = ?,
					sic = ?,
					website = ?
				WHERE
					cik = ?
			"#
		).bind(
			business_city
		).bind(
			business_country
		).bind(
			business_state
		).bind(
			business_street1
		).bind(
			business_zip
		).bind(
			description
		).bind(
			ein
		).bind(
			entity_type
		).bind(
			phone
		).bind(
			sic
		).bind(
			website
		).bind(
			cik
		).execute(
			self.db_connection.pool()
		).await?;

		// Fetch and return the updated row
		let security = sqlx::query_as::<_, SecurityRow>(
			"SELECT * FROM security WHERE cik = ?"
		).bind(
	   		cik
		).fetch_one(
			self.db_connection.pool()
		).await?;

		Ok(security)
	}
}
