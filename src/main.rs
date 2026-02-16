mod database;
mod handler;
mod schema;

use clap::Parser;
use dotenvy::dotenv;
use tokio::time::sleep;

use chrono::{Local};

use crate::handler::HandlerSecurityProfile;
use crate::handler::HandlerTime;
use crate::handler::handler_time::Seconds;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args
{
	/// Run the task immediately instead of waiting for 4am
	#[arg(long)]
	run_now: bool,

	/// Logging level (0 = none, 1 = info, 2 = debug)
	#[arg(long, default_value_t = 1)]
	log_level: u8,
}


/**
* Main function to run the security profile builder
*/
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
	println!("[INFO] Security Profile Builder starting up at {}", Local::now().format("%Y-%m-%d %H:%M:%S"));

	dotenv().ok();

	let args: Args = Args::parse();

	let handler_security_profile: HandlerSecurityProfile = HandlerSecurityProfile::new();

	if args.run_now
	{
		println!("[INFO] Running task immediately due to --run-now flag");

		if let Err(e) = handler_security_profile.synchronize(args.log_level).await
		{
			eprintln!("[ERROR] Error during immediate execution: {}", e);

			return Err(e);
		}

		println!("[INFO] Immediate execution completed. Exiting now <3");

		return Ok(());
	}

	let time_handler = HandlerTime::new();

	loop
	{
		let initial_delay: Seconds = time_handler.calculate_seconds_until_next_4am();

		println!(
			"[INFO] Calculated time until next 4am execution: {}h {}m {}s",
			initial_delay.as_secs() / 3600,
			(initial_delay.as_secs() % 3600) / 60,
			initial_delay.as_secs() % 60
		);

		sleep(initial_delay).await;

		if let Err(e) = handler_security_profile.synchronize(args.log_level).await
		{
			eprintln!("[ERROR] Error during execution: {}", e);

			return Err(e);
		}
	}
}
