use reqwest::blocking::Client;
use std::io;
use std::path::PathBuf;

use std::fs::{ File, Metadata };
use std::time::{ Duration, SystemTime };

use crate::handler::file::zip::HandlerCompanyfactsZip;
use crate::handler::file::zip::HandlerSubmissionsZip;

use crate::{ log_info };


pub struct UpdatedSecCompanyfactsAndSubmissions
{
	pub handler_submissions_zip: HandlerSubmissionsZip,
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


	/**
	* @visibility: Internal
	* Download the company facts zip
	*/
	fn download_companyfacts_zip(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!("Downloading SEC companyfacts.zip..");

		// Create the directory if it doesn't exist
		std::fs::create_dir_all(&self.path_dir_tmp)?;

		let client = Client::builder().user_agent(self.user_agent).build().expect("failed to build reqwest client");

		let mut response = client.get(self.request_url_companyfacts_zip).send()?;

		response.error_for_status_ref()?;

		let mut output: File = File::create(&self.path_dir_tmp.join(Self::COMPANY_FACTS_ZIP))?;

		io::copy(&mut response, &mut output)?;

		log_info!("Saved to {}", &self.path_dir_tmp.join(Self::COMPANY_FACTS_ZIP).display());

		Ok(())
	}

	/**
	* @visibility: Internal
	* Download the submissions zip
	*/
	fn download_submissions_zip(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!("Downloading SEC submissions.zip..");

		// Create the directory if it doesn't exist
		std::fs::create_dir_all(&self.path_dir_tmp)?;

		let client = Client::builder().user_agent(self.user_agent).build().expect("failed to build reqwest client");

		let mut response = client.get(self.request_url_submissions_zip).send()?;

		response.error_for_status_ref()?;

		let mut output: File = File::create(&self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP))?;

		io::copy(&mut response, &mut output)?;

		log_info!("Saved to {}", self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP).display());

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
				log_info!("companyfacts.zip is nonexistant.");

				return Ok(true);
			}
		};

		let modified: SystemTime = metadata.modified()?;

		let age: Duration = SystemTime::now().duration_since(modified)?;

		if age > Self::SECONDS_24_HOURS
		{
			log_info!("companyfacts.zip is old. Redownload needed.");

			return Ok(true);
		}
		else
		{
			log_info!("companyfacts.zip has age of {} seconds. No need to redownload.", age.as_secs());

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
				log_info!("submissions.zip is nonexistant.");

				return Ok(true);
			}
		};

		let modified: SystemTime = metadata.modified()?;

		let age: Duration = SystemTime::now().duration_since(modified)?;

		if age > Self::SECONDS_24_HOURS
		{
			log_info!("submissions.zip is old. Redownload needed.");

			return Ok(true);
		}
		else
		{
			log_info!("submissions.zip has age of {} seconds. No need to redownload.", age.as_secs());

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
			log_info!("companyfacts.zip file is up-to-date. Skipping download.");
		}

		if self.should_download_submissions_zip()?
		{
			self.download_submissions_zip()?;
		}
		else
		{
			log_info!("submissions.zip file is up-to-date. Skipping download.");
		}

		log_info!("{} exists, initializing a HandlerCompanyfactsZip for it..", Self::COMPANY_FACTS_ZIP);

		let handler_companyfacts_zip = HandlerCompanyfactsZip::new(self.path_dir_tmp.join(Self::COMPANY_FACTS_ZIP))?;

		log_info!("{} exists, initializing a HandlerCompanyfactsZip for it..", Self::SUBMISSIONS_ZIP);

		let handler_submissions_zip = HandlerSubmissionsZip::new(self.path_dir_tmp.join(Self::SUBMISSIONS_ZIP))?;

		Ok(
			UpdatedSecCompanyfactsAndSubmissions
			{
				handler_companyfacts_zip,
				handler_submissions_zip,
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
