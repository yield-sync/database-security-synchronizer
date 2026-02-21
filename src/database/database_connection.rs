use sqlx::MySqlPool;

use crate::{ log_info };


pub struct DatabaseConnection
{
	pool: MySqlPool,
}


impl DatabaseConnection
{
	pub async fn new() -> Result<Self, Box<dyn std::error::Error>>
	{
		let database_url = std::env::var("APP__DATABASE__URL").map_err(
			|_| "APP__DATABASE__URL environment variable not set"
		)?;

		log_info!("Creating connection to Database..");

		let pool = MySqlPool::connect(&database_url).await?;

		log_info!("Database connection established");

		Ok(Self { pool })
	}

	pub async fn close(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!("Closing connection to Database..");

		self.pool.close().await;

		Ok(())
	}

	pub fn pool(&self) -> &MySqlPool
	{
		&self.pool
	}
}
