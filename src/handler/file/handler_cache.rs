use serde_json::Value;
use std::path::PathBuf;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};


pub struct HandlerCache
{
	path: PathBuf,
}


impl HandlerCache
{
	pub fn new() -> Self
	{
		// Create the directory if it doesn't exist
		let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp");

		if !dir.exists()
		{
			std::fs::create_dir_all(&dir).expect("Failed to create directory");
		}

		// Path to the cache file
		let path = dir.join("cache.json");

		// Only create and initialize the file if it doesn't exist
		if !path.exists()
		{
			println!("cache.json not found, Creating now..");

			// creates only if it doesn't exist
			let mut file = OpenOptions::new().write(true).create(true).open(&path).expect(
				"Failed to create cache file"
			);

			let json = serde_json::json!({ "submission-file-with-no-tickers": [] });

			file.write_all(json.to_string().as_bytes()).expect("Failed to write to cache file");

			println!("cache.json created successfully");
		}

		Self { path }
	}

	/**
	* Adds a tickerless submission file name to the cache.
	*/
	pub fn add_tickerless_submission_file_name(&mut self, submission_file_name: &str)
	{
		// Open the file for read + write
		let mut file = OpenOptions::new().read(true).write(true).open(&self.path).expect(
			"Failed to open cache file"
		);

		// Read the existing JSON
		let mut content = String::new();

		file.read_to_string(&mut content).expect("Failed to read cache file");

		let mut json: Value = if content.trim().is_empty()
		{
			serde_json::json!({ "submission-file-with-no-tickers": [] })
		}
		else
		{
			serde_json::from_str(&content).expect("Failed to parse JSON from cache file")
		};

		// Append the new submission_file_name
		let tickerless_securities = json.get_mut("submission-file-with-no-tickers").expect(
		   	"Missing submission-file-with-no-tickers key"
		);

		tickerless_securities.as_array_mut().expect("submission-file-with-no-tickers is not an array").push(
		   	serde_json::Value::String(submission_file_name.to_string())
		);

		let array = tickerless_securities.as_array_mut().expect("submission-file-with-no-tickers is not an array");

		if !array.iter().any(|v| v.as_str() == Some(submission_file_name))
		{
			println!("Caching {} to submission-file-with-no-tickers", submission_file_name);

			array.push(serde_json::Value::String(submission_file_name.to_string()));
		}

		// Truncate and write updated JSON
		file.set_len(0).expect("Failed to truncate cache file");

		file.seek(SeekFrom::Start(0)).expect("Failed to seek to start of cache file");

		file.write_all(json.to_string().as_bytes()).expect("Failed to write updated JSON");
	}

	/**
	* Checks if a submission file is tickerless.
	*/
	pub fn is_tickerless_submission_file(&mut self, submission_file_name: &str) -> bool
	{
		let mut file = File::open(&self.path).expect("Failed to open cache file");

		// Read the existing JSON
		let mut content = String::new();

		file.read_to_string(&mut content).expect("Failed to read cache file");

		let json: Value = if content.trim().is_empty()
		{
			serde_json::json!({ "submission-file-with-no-tickers": [] })
		}
		else
		{
			serde_json::from_str(&content).expect("Failed to parse JSON from cache file")
		};

		let tickerless_securities = json.get("submission-file-with-no-tickers").expect(
		   	"Missing submission-file-with-no-tickers key"
		);

		tickerless_securities.as_array().expect("submission-file-with-no-tickers is not an array").iter().any(
			|v| v.as_str() == Some(submission_file_name)
		)
	}
}
