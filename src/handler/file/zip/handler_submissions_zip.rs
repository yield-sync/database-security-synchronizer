use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use zip::ZipArchive;

use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime};
use sha2::{Digest, Sha256};
use std::fs::{File};


use crate::schema::{SubmissionsData, SubmissionsDataFilings};


pub struct SubmissionsZipHandler
{
	path: PathBuf,
	archive: ZipArchive<File>,
}


impl SubmissionsZipHandler
{
	pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>>
	{
		let file = File::open(&path)?;

		let archive = ZipArchive::new(file)?;

		Ok(Self { path, archive, })
	}


	fn parse_acceptance_datetime(&self, raw_acceptance: &str) -> NaiveDateTime
	{
		DateTime::parse_from_rfc3339(raw_acceptance).ok().map(|dt| dt.with_timezone(&Utc).naive_utc()).unwrap()
	}

	fn parse_date(&self, raw_date: &str) -> Option<NaiveDate>
	{
		NaiveDate::parse_from_str(raw_date, "%Y-%m-%d").ok()
	}


	/**
	* Read all files directly from submissions.zip (NO extraction)
	* Returns HashMap<file_name, sha256_hash>
	*/
	pub fn compute_file_names_to_hashes(&mut self,) -> Result<HashMap<String, String>, Box<dyn std::error::Error>>
	{
		println!("Reading {} file names and computing file hashes..", self.path.display());

		let mut results: HashMap<String, String> = HashMap::new();
		let mut buffer: [u8; 8192] = [0u8; 8192];

		for i in 0..self.archive.len()
		{
			let mut zipped_file = self.archive.by_index(i)?;

			if zipped_file.is_dir()
			{
				continue;
			}

			let name = zipped_file.name().to_string();
			let mut hasher = Sha256::new();

			loop
			{
				let bytes_read = zipped_file.read(&mut buffer)?;

				if bytes_read == 0
				{
					break;
				}

				hasher.update(&buffer[..bytes_read]);
			}

			let hash = format!("{:x}", hasher.finalize());

			results.insert(name, hash); // insert into HashMap
		}

		Ok(results)
	}


	/**
	* Load JSON from a file inside submissions.zip
	*/
	pub fn load_json_from_file(&mut self, file_name: &str,) -> Result<Value, Box<dyn std::error::Error>>
	{
		// ZipArchive requires mutable access because reading advances internal cursor
		let mut zipped_file = self.archive.by_name(file_name)?;

		if zipped_file.is_dir()
		{
			return Err(format!("{} is a directory", file_name).into());
		}

		// Deserialize directly from the file reader (streaming)
		let value: Value = serde_json::from_reader(&mut zipped_file)?;

		Ok(value)
	}

	pub fn extract_submissions_data(
		&mut self,
		file_name: &str
	) -> Result<SubmissionsData, Box<dyn std::error::Error>>
	{
		let get_vec = |v: Option<&Value>| -> Vec<String> {
			v.and_then(|v| v.as_array()).map(|arr| {
				arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
			}).unwrap_or_default()
		};

		let json_submission: Value = self.load_json_from_file(file_name)?;

	 	let business = json_submission.get("addresses").and_then(|a| a.get("business"));

		let get_str = |v: Option<&Value>| {
			v.and_then(|v| v.as_str()).unwrap_or("<unknown>").to_string()
		};

		let tickers: Vec<String> = json_submission.get("tickers").and_then(|v| v.as_array()).filter(
			|arr| !arr.is_empty()
		).map(
			|arr|
			{
				arr.iter().map(
					|v|
					{
						if let Some(s) = v.as_str()
						{
							s.to_string()
						}
						else if v.is_null()
						{
							"null".to_string()
						}
						else
						{
							// Optional: decide what to do with non-string, non-null values
							v.to_string()
						}
					}
				).collect()
			}
		).unwrap_or_default();

		let exchanges: Vec<String> = json_submission.get("exchanges").and_then(|v| v.as_array()).filter(
			|arr| !arr.is_empty()
		).map(
			|arr|
			{
				arr.iter().map(
					|v|
					{
						if let Some(s) = v.as_str()
						{
							s.to_string()
						}
						else if v.is_null()
						{
							"null".to_string()
						}
						else
						{
							// Optional: decide what to do with non-string, non-null values
							v.to_string()
						}
					}
				).collect()
			}
		).unwrap_or_default();

		let recent = json_submission.get("filings").and_then(|v| v.get("recent"));

		let accession_number = get_vec(recent.and_then(|v| v.get("accessionNumber")));
		let filing_dates = get_vec(recent.and_then(|v| v.get("filingDate")));
		let report_dates = get_vec(recent.and_then(|v| v.get("reportDate")));
		let forms = get_vec(recent.and_then(|v| v.get("form")));
		let acceptance = get_vec(recent.and_then(|v| v.get("acceptanceDateTime")));

		let filings_len = accession_number.len();

		let mut filings = Vec::with_capacity(filings_len);

		for i in 0..filings_len
		{
			let filing_date = filing_dates.get(i).and_then(|s| self.parse_date(s)).ok_or(
				"Missing or invalid filing_date"
			)?;

			let report_date = report_dates.get(i).and_then(
				|s| {
					if s.is_empty()
					{
						None
					}
					else
					{
						self.parse_date(s)
					}
				}
			);

			let acceptance_dt = acceptance.get(i).ok_or("Missing acceptanceDateTime")?;

			let acceptance_dt = self.parse_acceptance_datetime(acceptance_dt);

			filings.push(
				SubmissionsDataFilings {
					accession_number: accession_number.get(i).cloned().unwrap_or_default(),
					filing_date,
					report_date,
					form: forms.get(i).cloned().unwrap_or_default(),
					acceptance: acceptance_dt
				}
			);
		}

		Ok(
			SubmissionsData {
				business_street1: get_str(business.and_then(|m| m.get("street1"))),
				business_city: get_str(business.and_then(|m| m.get("city"))),
				business_state: get_str(business.and_then(|m| m.get("stateOrCountry"))),
				business_country: get_str(business.and_then(|m| m.get("country"))),
				business_zip: get_str(business.and_then(|m| m.get("zipCode"))),
				cik: get_str(json_submission.get("cik")),
				description: get_str(json_submission.get("description")),
				ein: get_str(json_submission.get("ein")),
				entity_type: get_str(json_submission.get("entityType")),
				phone: get_str(json_submission.get("phone")),
				name: get_str(json_submission.get("name")),
				sic: get_str(json_submission.get("sic")),
				website: get_str(json_submission.get("website")),
				tickers,
				exchanges,
				filings,
			}
		)
	}
}
