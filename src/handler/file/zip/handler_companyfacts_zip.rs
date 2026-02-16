use serde_json::Value;
use std::path::PathBuf;
use zip::ZipArchive;

use std::fs::{File};

use crate::schema::{CompanyfactsCommonStockSharesOutstanding, Companyfacts};


pub struct HandlerCompanyfactsZip
{
	archive: ZipArchive<File>,
}


impl HandlerCompanyfactsZip
{
	pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>>
	{
		let file = File::open(&path)?;

		let archive = ZipArchive::new(file)?;

		Ok(Self { archive, })
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

	pub fn extract_data(&mut self, file_name: &str) -> Result<Companyfacts, Box<dyn std::error::Error>>
	{
		let json_submission: Value = self.load_json_from_file(file_name)?;

		let shares_outstanding: Vec<CompanyfactsCommonStockSharesOutstanding> = json_submission.get("facts").and_then(
			|v| v.get("us-gaap")
		).and_then(
			|v| v.get("CommonStockSharesOutstanding")
		).and_then(
			|v| v.get("units")
		).and_then(
			|v| v.get("shares")
		).and_then(
			|v| v.as_array()
		).filter(
			|arr| !arr.is_empty()
		).map(
			|arr| {
				arr.iter().filter_map(|item| {
					Some(CompanyfactsCommonStockSharesOutstanding {
						security_filing_accession_number: item.get("accn")?.as_str()?.to_owned(),
						end: item.get("end")?.as_str()?.to_owned(),
						filed: item.get("filed")?.as_str()?.to_owned(),
						fp: item.get("fp")?.as_str()?.to_owned(),
						fy: item.get("fy")?.as_i64()?,
						form: item.get("form")?.as_str()?.to_owned(),
						common_stock_shares_outstanding: item.get("val")?.as_i64()?,
					})
				}).collect()
			}
		).unwrap_or_default();

		Ok(
			Companyfacts {
				common_stock_shares_outstanding: shares_outstanding,
			}
		)
	}

	pub fn file_exists(&mut self, file_name: &str) -> bool
	{
		self.archive.by_name(file_name).is_ok()
	}
}
