use reqwest::Client;
use std::path::{ Path, PathBuf };

use std::fs::{ Metadata };
use std::time::{ Duration, SystemTime };

use tokio::fs::File;
use tokio::io::{ AsyncWriteExt, BufWriter };
use zip::ZipArchive;

use crate::handler::file::zip::HandlerFileCompanyfactsZip;
use crate::handler::file::zip::HandlerFileSubmissionsZip;

use crate::{ log_info };


pub struct UpdatedSecCompanyfactsAndSubmissions
{
	pub handler_file_submissions_zip: HandlerFileSubmissionsZip,
	pub handler_file_companyfacts_zip: HandlerFileCompanyfactsZip,
}

pub struct HandlerApiSec
{
	client: Client,
	path_dir_tmp: PathBuf,
	request_url_companyfacts_zip: &'static str,
	request_url_submissions_zip: &'static str,
}


impl HandlerApiSec
{
	pub const COMPANY_FACTS_ZIP: &'static str = "companyfacts.zip";
	pub const SUBMISSIONS_ZIP: &'static str = "submissions.zip";

	const USER_AGENT: &'static str = "https://www.yieldsync.xyz w3st.io2021@gmail.com";

	// Scheduled runs are 24h apart and the file timestamp is set when the download finishes, so a 24h limit would skip
	// every other day. 20h leaves room for download time while still blocking repeat downloads on quick restarts.
	const MAX_ZIP_AGE: Duration = Duration::from_secs(20 * 60 * 60);

	// Fail fast if the SEC can't be reached at all, but give the full download plenty of time since the zips are large
	const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
	const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(60 * 60);

	// Buffer writes to disk so each small network chunk doesn't become its own file write
	const WRITE_BUFFER_BYTES: usize = 1024 * 1024;

	const PROGRESS_LOG_INTERVAL_BYTES: u64 = 250 * 1024 * 1024;


	/**
	* @visibility: Public
	* Constructor for SecurityProfileBuilder
	*/
	pub fn new() -> Self
	{
		Self
		{
			client: Client::builder().user_agent(Self::USER_AGENT).connect_timeout(
				Self::CONNECT_TIMEOUT
			).timeout(
				Self::DOWNLOAD_TIMEOUT
			).build().expect(
				"failed to build reqwest client"
			),
			// @dev use /tmp/handler_api_sec on production but for development set TMP_DIR
			path_dir_tmp: PathBuf::from(
				std::env::var("TMP_DIR").unwrap_or_else(|_| "/tmp/handler_api_sec".to_string())
			),
			request_url_companyfacts_zip: "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip",
			request_url_submissions_zip: "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip",
		}
	}


	/**
	* @visibility: Internal
	* Opens the zip's central directory to make sure the file is a complete, readable zip
	*/
	fn verify_zip(path: &Path) -> Result<(), Box<dyn std::error::Error>>
	{
		let file = std::fs::File::open(path)?;

		ZipArchive::new(file)?;

		Ok(())
	}

	/**
	* @visibility: Internal
	* Stream a zip from the SEC to disk.
	*
	* The download is written to "<name>.partial" and only renamed over the real file once it is complete and
	* verified. A crash or failed download therefore never replaces a good zip with a broken one, and the rename is
	* what sets the timestamp used by should_download.
	*/
	async fn download_zip(&self, url: &str, file_name: &str) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!("Downloading SEC {}..", file_name);

		// Create the directory if it doesn't exist
		tokio::fs::create_dir_all(&self.path_dir_tmp).await?;

		let path_final: PathBuf = self.path_dir_tmp.join(file_name);
		let path_partial: PathBuf = self.path_dir_tmp.join(format!("{}.partial", file_name));

		let mut response = self.client.get(url).send().await?.error_for_status()?;

		let mut writer = BufWriter::with_capacity(Self::WRITE_BUFFER_BYTES, File::create(&path_partial).await?);

		let mut bytes_written: u64 = 0;
		let mut next_progress_log: u64 = Self::PROGRESS_LOG_INTERVAL_BYTES;

		// Read the body one chunk at a time so the whole zip is never held in memory
		while let Some(chunk) = response.chunk().await?
		{
			writer.write_all(&chunk).await?;

			bytes_written += chunk.len() as u64;

			if bytes_written >= next_progress_log
			{
				log_info!("{}: {} MB downloaded..", file_name, bytes_written / (1024 * 1024));

				next_progress_log += Self::PROGRESS_LOG_INTERVAL_BYTES;
			}
		}

		// Make sure every byte is actually on disk before the file is trusted
		writer.flush().await?;
		writer.get_ref().sync_all().await?;

		drop(writer);

		if let Err(e) = Self::verify_zip(&path_partial)
		{
			let _ = tokio::fs::remove_file(&path_partial).await;

			return Err(format!("Downloaded {} is not a valid zip: {}", file_name, e).into());
		}

		tokio::fs::rename(&path_partial, &path_final).await?;

		log_info!("Saved {} MB to {}", bytes_written / (1024 * 1024), path_final.display());

		Ok(())
	}

	/**
	* @visibility: Internal
	* Checks if a zip is missing or older than MAX_ZIP_AGE
	*/
	fn should_download_zip(&self, file_name: &str) -> Result<bool, Box<dyn std::error::Error>>
	{
		let path_zip: PathBuf = self.path_dir_tmp.join(file_name);

		let metadata: Metadata = match std::fs::metadata(&path_zip)
		{
			Ok(m) => m,
			Err(_) =>
			{
				log_info!("{} is nonexistent.", file_name);

				return Ok(true);
			}
		};

		let modified: SystemTime = metadata.modified()?;

		let age: Duration = SystemTime::now().duration_since(modified)?;

		if age > Self::MAX_ZIP_AGE
		{
			log_info!("{} is {} seconds old. Redownload needed.", file_name, age.as_secs());

			return Ok(true);
		}
		else
		{
			log_info!("{} has age of {} seconds. No need to redownload.", file_name, age.as_secs());

			return Ok(false);
		}
	}


	/**
	* @visibility: Public
	* Run full update
	*/
	pub async fn get_updated_companyfacts_and_submissions(
		&self
	) -> Result<UpdatedSecCompanyfactsAndSubmissions, Box<dyn std::error::Error>>
	{
		if self.should_download_zip(Self::COMPANY_FACTS_ZIP)?
		{
			self.download_zip(self.request_url_companyfacts_zip, Self::COMPANY_FACTS_ZIP).await?;
		}
		else
		{
			log_info!("{} file is up-to-date. Skipping download.", Self::COMPANY_FACTS_ZIP);
		}

		if self.should_download_zip(Self::SUBMISSIONS_ZIP)?
		{
			self.download_zip(self.request_url_submissions_zip, Self::SUBMISSIONS_ZIP).await?;
		}
		else
		{
			log_info!("{} file is up-to-date. Skipping download.", Self::SUBMISSIONS_ZIP);
		}

		log_info!("{} exists, initializing a HandlerFileCompanyfactsZip for it..", Self::COMPANY_FACTS_ZIP);

		let handler_file_companyfacts_zip = HandlerFileCompanyfactsZip::new(
			self.path_dir_tmp.join(Self::COMPANY_FACTS_ZIP)
		)?;

		log_info!("{} exists, initializing a HandlerFileSubmissionsZip for it..", Self::SUBMISSIONS_ZIP);

		let handler_file_submissions_zip = HandlerFileSubmissionsZip::new(
			self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP)
		)?;

		Ok(
			UpdatedSecCompanyfactsAndSubmissions
			{
				handler_file_companyfacts_zip,
				handler_file_submissions_zip,
			}
		)
	}
}


impl Default for HandlerApiSec
{
	fn default() -> Self
	{
		Self::new()
	}
}
