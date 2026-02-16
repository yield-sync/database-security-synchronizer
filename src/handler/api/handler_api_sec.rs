use reqwest::blocking::Client;
use std::fs;
use std::io;
use std::path::PathBuf;

use std::fs::{File, Metadata};
use std::time::{Duration, SystemTime};

use crate::handler::file::zip::HandlerCompanyfactsZip;
use crate::handler::file::zip::SubmissionsZipHandler;


pub struct UpdatedSecCompanyfactsAndSubmissions
{
	pub submissions_zip_handler: SubmissionsZipHandler,
	pub handler_previous_submissions_zip: Option<SubmissionsZipHandler>,

	pub handler_companyfacts_zip: HandlerCompanyfactsZip,
}

pub struct HandlerApiSec
{
	path_dir_tmp: PathBuf,
	user_agent: &'static str,
	request_url_companyfacts_zip: &'static str,
	request_url_submissions_zip: &'static str,
}


impl HandlerApiSec
{
	pub const COMPANY_FACTS_ZIP: &'static str = "companyfacts.zip";
	pub const PREVIOUS_SUBMISSIONS_ZIP: &'static str = "previous_submissions.zip";
	pub const SUBMISSIONS_ZIP: &'static str = "submissions.zip";

	const SECONDS_24_HOURS: Duration = Duration::from_secs(24 * 60 * 60);


	/**
	* @visibility: Public
	* Constructor for SecurityProfileBuilder
	*/
	pub fn new() -> Self
	{
		Self
		{
			path_dir_tmp: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp"),
			user_agent: "https://www.yieldsync.xyz w3st.io2021@gmail.com",
			request_url_companyfacts_zip: "https://www.sec.gov/Archives/edgar/daily-index/xbrl/companyfacts.zip",
			request_url_submissions_zip: "https://www.sec.gov/Archives/edgar/daily-index/bulkdata/submissions.zip",
		}
	}


	fn archive_current_submissions_zip(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		let current_path: PathBuf = self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP);

		let previous_path: PathBuf = self.path_dir_tmp.join(Self::PREVIOUS_SUBMISSIONS_ZIP);

		if !current_path.exists()
		{
			println!("No current submissions.zip exists. Skipping archive process..");

			return Ok(());
		}

		println!("Renaming submissions.zip to previous_submissions.zip..");

		fs::copy(&current_path, &previous_path)?;

		Ok(())
	}

	/**
	* @visibility: Internal
	* Download the company facts zip
	*/
	fn download_companyfacts_zip(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		println!("Downloading SEC companyfacts.zip..");

		// Create the directory if it doesn't exist
		std::fs::create_dir_all(&self.path_dir_tmp)?;

		let client = Client::builder().user_agent(self.user_agent).build().expect("failed to build reqwest client");

		let mut response = client.get(self.request_url_companyfacts_zip).send()?;

		response.error_for_status_ref()?;

		let mut output: File = File::create(&self.path_dir_tmp.join(Self::COMPANY_FACTS_ZIP))?;

		io::copy(&mut response, &mut output)?;

		println!("Saved to {}", &self.path_dir_tmp.join(Self::COMPANY_FACTS_ZIP).display());

		Ok(())
	}

	/**
	* @visibility: Internal
	* Download the submissions zip
	*/
	fn download_submissions_zip(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		println!("Downloading SEC submissions.zip..");

		// Create the directory if it doesn't exist
		std::fs::create_dir_all(&self.path_dir_tmp)?;

		let client = Client::builder().user_agent(self.user_agent).build().expect("failed to build reqwest client");

		let mut response = client.get(self.request_url_submissions_zip).send()?;

		response.error_for_status_ref()?;

		let mut output: File = File::create(&self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP))?;

		io::copy(&mut response, &mut output)?;

		println!("Saved to {}", self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP).display());

		Ok(())
	}

	/**
	* @visibility: Internal
	* Checks if the company facts zip file is older than 24 hours.
	*/
	fn should_download_companyfacts_zip(&self) -> Result<bool, Box<dyn std::error::Error>>
	{
		let path_companyfacts_zip: PathBuf = self.path_dir_tmp.join(Self::COMPANY_FACTS_ZIP);

		let metadata: Metadata = match std::fs::metadata(&path_companyfacts_zip)
		{
			Ok(m) => m,
			Err(_) =>
			{
				println!("companyfacts.zip is nonexistant.");

				return Ok(true);
			}
		};

		let modified: SystemTime = metadata.modified()?;

		let age: Duration = SystemTime::now().duration_since(modified)?;

		if age > Self::SECONDS_24_HOURS
		{
			println!("companyfacts.zip is old. Redownload needed.");

			return Ok(true);
		}
		else
		{
			println!("companyfacts.zip has age of {} seconds. No need to redownload.", age.as_secs());

			return Ok(false);
		}
	}

	/**
	* @visibility: Internal
	* Checks if the submissions zip file is older than 24 hours.
	*/
	fn should_download_submissions_zip(&self) -> Result<bool, Box<dyn std::error::Error>>
	{
		let path_submissions_zip: PathBuf = self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP);

		let metadata: Metadata = match std::fs::metadata(&path_submissions_zip)
		{
			Ok(m) => m,
			Err(_) =>
			{
				println!("submissions.zip is nonexistant.");

				return Ok(true);
			}
		};

		let modified: SystemTime = metadata.modified()?;

		let age: Duration = SystemTime::now().duration_since(modified)?;

		if age > Self::SECONDS_24_HOURS
		{
			println!("submissions.zip is old. Redownload needed.");

			return Ok(true);
		}
		else
		{
			println!("submissions.zip has age of {} seconds. No need to redownload.", age.as_secs());

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
		if self.should_download_companyfacts_zip()?
		{
			self.download_companyfacts_zip()?;
		}
		else
		{
			println!("companyfacts.zip file is up-to-date. Skipping download.");
		}

		if self.should_download_submissions_zip()?
		{
			let path_current_submissions_zip: PathBuf = self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP);

			if path_current_submissions_zip.exists()
			{
				self.archive_current_submissions_zip()?;
			}

			self.download_submissions_zip()?;
		}
		else
		{
			println!("submissions.zip file is up-to-date. Skipping download.");
		}

		println!("{} exists, initializing a HandlerCompanyfactsZip for it..", Self::COMPANY_FACTS_ZIP);

		let handler_companyfacts_zip = HandlerCompanyfactsZip::new(self.path_dir_tmp.join(Self::COMPANY_FACTS_ZIP))?;

		println!("{} exists, initializing a HandlerCompanyfactsZip for it..", Self::SUBMISSIONS_ZIP);

		let submissions_zip_handler = SubmissionsZipHandler::new(self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP))?;

		let handler_previous_submissions_zip = {
			let previous_path = self.path_dir_tmp.join(Self::PREVIOUS_SUBMISSIONS_ZIP);

			if previous_path.exists()
			{
				println!("{} exists, initializing a SubmissionsZipHandler for it..", Self::PREVIOUS_SUBMISSIONS_ZIP);
				Some(SubmissionsZipHandler::new(previous_path)?)
			}
			else
			{
				println!("{} does not exist", Self::PREVIOUS_SUBMISSIONS_ZIP);
				None
			}
		};

		Ok(
			UpdatedSecCompanyfactsAndSubmissions
			{
				handler_companyfacts_zip,
				submissions_zip_handler,
				handler_previous_submissions_zip,
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
