use std::time::Duration;

use chrono::{DateTime, Local, NaiveTime};


pub type Seconds = std::time::Duration;


pub struct HandlerTime
{}


impl HandlerTime
{
	/**
	* @visibility: Public
	*/
	pub fn new() -> Self
	{
		Self
		{}
	}


	/**
	* Calculate the time until the next 4am and return it as a Duration in seconds
	*/
	pub fn calculate_seconds_until_next_4am(&self) -> Seconds
	{
		let now: DateTime<Local> = Local::now();
		let target_time = NaiveTime::from_hms_opt(4, 0, 0).expect("Valid time");

		let today_4am = now.date_naive().and_time(target_time);
		let tomorrow_4am = (now.date_naive() + chrono::Duration::days(1)).and_time(target_time);

		let target_datetime = if now.time() < target_time
		{
			// If it's before 4am today, schedule for today at 4am
			today_4am
		}
		else
		{
			// If it's after 4am today, schedule for tomorrow at 4am
			tomorrow_4am
		};

		let target_local = target_datetime.and_local_timezone(Local).unwrap();
		let duration_until_target = target_local.signed_duration_since(now);

		return Duration::from_secs(duration_until_target.num_seconds() as u64);
	}
}
