mod logger;
mod database;
mod handler;
mod schema;

use std::error::Error;
use std::sync::Arc;

use chrono::Local;
use clap::Parser;
use dotenvy::dotenv;
use tokio::signal::unix::{ signal, SignalKind };
use tokio::time::sleep;

use crate::database::database_connection::DatabaseConnection;
use crate::handler::{ HandlerDatabaseSecuritySynchronizer, HandlerTime };

type AppResult<T> = Result<T, Box<dyn Error>>;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args
{
	// Run a synchronization immediately, then continue on the daily 4am schedule
	#[arg(long)]
	run_now: bool,

	// Run a single synchronization and exit (for cron / systemd timers)
	#[arg(long)]
	once: bool,
}


// Entry point: sets up resources, runs the scheduler, and always closes the DB.
#[tokio::main]
async fn main() -> AppResult<()>
{
	dotenv().ok();

	let args = Args::parse();

	log_info!("Security Profile Builder starting up at {}", Local::now().format("%Y-%m-%d %H:%M:%S"));

	let db_connection = Arc::new(DatabaseConnection::new().await?);

	let handler_database_security_synchronizer = HandlerDatabaseSecuritySynchronizer::new(db_connection.clone()).await?;

	let result = run(&args, &handler_database_security_synchronizer).await;

	if let Err(e) = &result
	{
		log_error!("Fatal error: {}", e);
	}

	// Cleanup happens exactly once, and a close failure never masks the original error
	if let Err(e) = db_connection.close().await
	{
		log_error!("Failed to close database connection: {}", e);
	}

	result
}

// Runs the optional immediate sync, then the daily 4am loop until a shutdown signal.
async fn run(args: &Args, handler_database_security_synchronizer: &HandlerDatabaseSecuritySynchronizer) -> AppResult<()>
{
	if args.run_now || args.once
	{
		log_info!("Running synchronization immediately");

		handler_database_security_synchronizer.synchronize().await?;

		log_info!("Immediate execution completed");

		if args.once
		{
			return Ok(());
		}
	}

	let mut sigterm = signal(SignalKind::terminate())?;

	loop
	{
		let delay = HandlerTime::new().calculate_seconds_until_next_4am();
		let secs = delay.as_secs();

		log_info!("Next execution in {}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60);

		tokio::select!
		{
			_ = sleep(delay) => {}
			_ = tokio::signal::ctrl_c() =>
			{
				log_info!("Received Ctrl+C, shutting down");
				return Ok(());
			}
			_ = sigterm.recv() =>
			{
				log_info!("Received SIGTERM, shutting down");
				return Ok(());
			}
		}

		match handler_database_security_synchronizer.synchronize().await
		{
			Ok(_) => log_info!("Scheduled synchronization completed"),
			Err(e) => log_error!("Scheduled synchronization failed: {} (will retry at next 4am)", e),
		}
	}
}
