use serde_json::Value;
use std::path::PathBuf;
use zip::ZipArchive;

use std::fs::{ File };

use crate::schema::{
	Companyfacts,
	CompanyfactsCommonStockSharesOutstanding,
	CompanyfactsEntityCommonStockSharesOutstanding,
};


pub struct HandlerFileCompanyfactsZip
{
	archive: ZipArchive<File>,
}


impl HandlerFileCompanyfactsZip
{
	fn extract_common_stock_shares_outstanding(
		&mut self,
		json_submission: &Value
	) -> Result<Vec<CompanyfactsCommonStockSharesOutstanding>, Box<dyn std::error::Error>>
	{
		let common_stock_shares_outstanding: Vec<CompanyfactsCommonStockSharesOutstanding> = json_submission.get(
			"facts"
		).and_then(
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
						val: item.get("val")?.as_i64()?,
					})
				}).collect()
			}
		).unwrap_or_default();

		Ok(common_stock_shares_outstanding)
	}

	fn extract_entity_common_stock_shares_outstanding(
		&mut self,
		json_submission: &Value
	) -> Result<Vec<CompanyfactsEntityCommonStockSharesOutstanding>, Box<dyn std::error::Error>>
	{
		let entity_common_stock_shares_outstanding: Vec<CompanyfactsEntityCommonStockSharesOutstanding> = json_submission.get(
			"facts"
		).and_then(
			|v| v.get("dei")
		).and_then(
			|v| v.get("EntityCommonStockSharesOutstanding")
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
					Some(CompanyfactsEntityCommonStockSharesOutstanding {
						security_filing_accession_number: item.get("accn")?.as_str()?.to_owned(),
						end: item.get("end")?.as_str()?.to_owned(),
						filed: item.get("filed")?.as_str()?.to_owned(),
						fp: item.get("fp")?.as_str()?.to_owned(),
						fy: item.get("fy")?.as_i64()?,
						form: item.get("form")?.as_str()?.to_owned(),
						entity_common_stock_shares_outstanding: item.get("val")?.as_i64()?,
					})
				}).collect()
			}
		).unwrap_or_default();

		Ok(entity_common_stock_shares_outstanding)
	}


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

		let common_stock_shares_outstanding: Vec<
			CompanyfactsCommonStockSharesOutstanding
		> = self.extract_common_stock_shares_outstanding(
			&json_submission
		)?;

		let entity_common_stock_shares_outstanding: Vec<
			CompanyfactsEntityCommonStockSharesOutstanding
		> = self.extract_entity_common_stock_shares_outstanding(
			&json_submission
		)?;

		Ok(
			Companyfacts {
				common_stock_shares_outstanding,
				entity_common_stock_shares_outstanding,
			}
		)
	}

	pub fn file_exists(&mut self, file_name: &str) -> bool
	{
		self.archive.by_name(file_name).is_ok()
	}
}
