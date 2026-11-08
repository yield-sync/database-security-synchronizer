mod logger;
mod database;
mod handler;
mod schema;

use std::error::Error;
use std::sync::Arc;
use std::time::{ Duration, Instant };

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
	/// Skip the startup synchronization and wait for the next 4am run
	#[arg(long, conflicts_with = "once")]
	skip_run_immediately: bool,

	/// Run a single synchronization and exit (for cron / systemd timers)
	#[arg(long)]
	once: bool,
}

const FORCE_SYNCHRONIZE_ON_STARTUP: bool = true;
const FORCE_SYNCHRONIZE_ON_SCHEDULE: bool = false;


// Formats a duration as "Xh Ym Z.mmms"
fn format_duration(duration: Duration) -> String
{
	let total_seconds = duration.as_secs();

	format!(
		"{}h {}m {}.{:03}s",
		total_seconds / 3600,
		(total_seconds % 3600) / 60,
		total_seconds % 60,
		duration.subsec_millis(),
	)
}


// Entry point: sets up resources, runs the scheduler, and always closes the DB.
#[tokio::main]
async fn main() -> AppResult<()>
{
	let process_start = Instant::now();

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

	log_info!("Security Profile Builder ran for {}", format_duration(process_start.elapsed()));

	result
}


// Runs the optional immediate sync, then the daily 4am loop until a shutdown signal.
async fn run(args: &Args, handler_database_security_synchronizer: &HandlerDatabaseSecuritySynchronizer) -> AppResult<()>
{
	if !args.skip_run_immediately
	{
		log_info!("Running synchronization immediately");

		let startup_start = Instant::now();
		let startup_result = handler_database_security_synchronizer.synchronize(FORCE_SYNCHRONIZE_ON_STARTUP).await;
		let startup_elapsed = format_duration(startup_start.elapsed());

		if args.once
		{
			match &startup_result
			{
				Ok(_) => log_info!("Single synchronization completed in {}", startup_elapsed),
				Err(_) => log_error!("Single synchronization failed after {}", startup_elapsed),
			}

			// Single run mode: pass the error up so the process exits with a failure code
			return startup_result;
		}

		match startup_result
		{
			Ok(_) => log_info!("Immediate execution completed in {}", startup_elapsed),
			Err(error) => log_error!(
				"Startup synchronization failed after {}: {} (will retry at next 4am)",
				startup_elapsed,
				error,
			),
		}
	}

	let mut sigterm = signal(SignalKind::terminate())?;

	loop
	{
		let delay = HandlerTime::new().calculate_seconds_until_next_4am();

		log_info!("Next execution in {}", format_duration(delay));

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

		let scheduled_start = Instant::now();
		let scheduled_result = handler_database_security_synchronizer.synchronize(FORCE_SYNCHRONIZE_ON_SCHEDULE).await;
		let scheduled_elapsed = format_duration(scheduled_start.elapsed());

		match scheduled_result
		{
			Ok(_) => log_info!("Scheduled synchronization completed in {}", scheduled_elapsed),
			Err(e) => log_error!(
				"Scheduled synchronization failed after {}: {} (will retry at next 4am)",
				scheduled_elapsed,
				e,
			),
		}
	}
}
