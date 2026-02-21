use serde_json::Value;
use std::path::PathBuf;

use std::fs::{ File, OpenOptions };
use std::io::{ Read, Seek, SeekFrom, Write };

use crate::{ log_debug, log_info };


pub struct HandlerCacheSubmissionsFileWithNoTickers
{
	path: PathBuf,
}


impl HandlerCacheSubmissionsFileWithNoTickers
{
	pub fn new() -> Self
	{
		// Create the directory if it doesn't exist
		let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp").join("cache");

		if !dir.exists()
		{
			std::fs::create_dir_all(&dir).expect("Failed to create directory");
		}

		let path = dir.join("submission-file-with-no-tickers.json");

		if !path.exists()
		{
			log_info!("submission-file-with-no-tickers.json not found, Creating now..");

			let mut file = OpenOptions::new().write(true).create(true).open(&path).expect(
				"Failed to create submission-file-with-no-tickers.json file"
			);

			let json = serde_json::json!([]);

			file.write_all(json.to_string().as_bytes()).expect(
				"Failed to write to submission-file-with-no-tickers.json file"
			);

			log_info!("submission-file-with-no-tickers.json created successfully");
		}

		Self { path }
	}

	/**
	* Adds a tickerless submission file name to the cache.
	*/
	pub fn add_tickerless_submission_file_name(&self, submission_file_name: &str)
	{
		let mut file = OpenOptions::new().read(true).write(true).open(&self.path).expect(
			"Failed to open submission-file-with-no-tickers.json"
		);

		// Read existing JSON
		let mut content = String::new();

		file.read_to_string(&mut content).expect("Failed to read file");

		let mut json: Value = if content.trim().is_empty()
		{
			serde_json::json!([])
		}
		else
		{
			serde_json::from_str(&content).expect("Failed to parse JSON from submission-file-with-no-tickers.json")
		};

		let array = json.as_array_mut().expect("submission-file-with-no-tickers is not an array");

		// Only push if not already present
		if !array.iter().any(|v| v.as_str() == Some(submission_file_name))
		{
			log_debug!("Caching {} to submission-file-with-no-tickers.json", submission_file_name);

			array.push(Value::String(submission_file_name.to_string()));
		}

		// Rewrite file
		file.set_len(0).expect("Failed to truncate submission-file-with-no-tickers.json");

		file.seek(SeekFrom::Start(0)).expect("Failed to seek to start of submission-file-with-no-tickers.json");

		file.write_all(json.to_string().as_bytes()).expect("Failed to write updated JSON");
	}

	/**
	* Checks if a submission file is tickerless.
	*/
	pub fn is_tickerless_submission_file(&mut self, submission_file_name: &str) -> bool
	{
		let mut file = File::open(&self.path).expect("Failed to open submission-file-with-no-tickers.json");

		let mut content = String::new();

		file.read_to_string(&mut content).expect("Failed to read submission-file-with-no-tickers.json");

		let json: Value = if content.trim().is_empty()
		{
			serde_json::json!([])
		}
		else
		{
			serde_json::from_str(&content).expect("Invalid JSON in submission-file-with-no-tickers.json")
		};

		json.as_array().expect("Cache JSON root must be an array").iter().any(
			|v| v.as_str() == Some(submission_file_name)
		)
	}
}
