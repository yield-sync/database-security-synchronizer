use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use zip::ZipArchive;

use std::fs::{ File };

use crate::{ log_debug };
use crate::schema::{
	Assets,
	Companyfacts,
	CommonStockSharesOutstanding,
	EntityCommonStockSharesOutstanding,
};


pub struct HandlerFileCompanyfactsZip
{
	archive: ZipArchive<File>,
}


impl HandlerFileCompanyfactsZip
{
	// No instance needed

	/**
	* Keep only the entry with the latest "end" for each accession number. A filing can report the same fact for
	* multiple periods (current and prior), but only the current period belongs to that filing.
	*/
	fn keep_latest_end_per_filing<T>(rows: Vec<T>, accn: fn(&T) -> &str, end: fn(&T) -> &str) -> Vec<T>
	{
		let mut latest: HashMap<String, T> = HashMap::new();

		for row in rows
		{
			let key: String = accn(&row).to_string();

			match latest.get(&key)
			{
				// "end" is YYYY-MM-DD so string comparison matches date order
				Some(existing) if end(existing) >= end(&row) => {}
				_ =>
				{
					latest.insert(key, row);
				}
			}
		}

		latest.into_values().collect()
	}

	pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>>
	{
		let file = File::open(&path)?;

		let archive = ZipArchive::new(file)?;

		Ok(Self { archive, })
	}


	// Instance methods
	fn extract_assets(&mut self, json_submission: &Value) -> Result<Vec<Assets>, Box<dyn std::error::Error>>
	{
		let entries: &[Value] = json_submission.get(
			"facts"
		).and_then(
			|v| v.get("us-gaap")
		).and_then(
			|v| v.get("Assets")
		).and_then(
			|v| v.get("units")
		).and_then(
			|v| v.get("USD")
		).and_then(
			|v| v.as_array()
		).map(
			|arr| arr.as_slice()
		).unwrap_or(&[]);

		let assets: Vec<Assets> = entries.iter().filter_map(
			|entry|
			{
				Some(
					Assets
					{
						filing_accession_number: entry.get("accn")?.as_str()?.to_owned(),
						end: entry.get("end")?.as_str()?.to_owned(),
						fp: entry.get("fp")?.as_str()?.to_owned(),
						fy: entry.get("fy")?.as_i64()?,
						val: entry.get("val")?.as_f64()?,
					}
				)
			}
		).collect();

		if assets.len() < entries.len()
		{
			log_debug!(
				"Assets: dropped {} of {} entries with missing or invalid fields",
				entries.len() - assets.len(),
				entries.len(),
			);
		}

		Ok(assets)
	}

	fn extract_common_stock_shares_outstanding(
		&mut self,
		json_submission: &Value,
	) -> Result<Vec<CommonStockSharesOutstanding>, Box<dyn std::error::Error>>
	{
		let entries: &[Value] = json_submission.get(
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
		).map(
			|arr| arr.as_slice()
		).unwrap_or(&[]);

		let common_stock_shares_outstanding: Vec<CommonStockSharesOutstanding> = entries.iter().filter_map(
			|entry|
			{
				Some(
					CommonStockSharesOutstanding
					{
						filing_accession_number: entry.get("accn")?.as_str()?.to_owned(),
						end: entry.get("end")?.as_str()?.to_owned(),
						fp: entry.get("fp")?.as_str()?.to_owned(),
						fy: entry.get("fy")?.as_i64()?,
						val: entry.get("val")?.as_f64()?,
					}
				)
			}
		).collect();

		if common_stock_shares_outstanding.len() < entries.len()
		{
			log_debug!(
				"CommonStockSharesOutstanding: dropped {} of {} entries with missing or invalid fields",
				entries.len() - common_stock_shares_outstanding.len(),
				entries.len(),
			);
		}

		Ok(
			Self::keep_latest_end_per_filing(
				common_stock_shares_outstanding,
				|r| &r.filing_accession_number,
				|r| &r.end,
			)
		)
	}

	fn extract_entity_common_stock_shares_outstanding(
		&mut self,
		json_submission: &Value,
	) -> Result<Vec<EntityCommonStockSharesOutstanding>, Box<dyn std::error::Error>>
	{
		let entries: &[Value] = json_submission.get(
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
		).map(
			|arr| arr.as_slice()
		).unwrap_or(&[]);

		let entity_common_stock_shares_outstanding: Vec<EntityCommonStockSharesOutstanding> = entries.iter().filter_map(
			|entry|
			{
				Some(
					EntityCommonStockSharesOutstanding
					{
						filing_accession_number: entry.get("accn")?.as_str()?.to_owned(),
						end: entry.get("end")?.as_str()?.to_owned(),
						fp: entry.get("fp")?.as_str()?.to_owned(),
						fy: entry.get("fy")?.as_i64()?,
						val: entry.get("val")?.as_f64()?,
					}
				)
			}
		).collect();

		if entity_common_stock_shares_outstanding.len() < entries.len()
		{
			log_debug!(
				"EntityCommonStockSharesOutstanding: dropped {} of {} entries with missing or invalid fields",
				entries.len() - entity_common_stock_shares_outstanding.len(),
				entries.len(),
			);
		}

		Ok(
			Self::keep_latest_end_per_filing(
				entity_common_stock_shares_outstanding,
				|r| &r.filing_accession_number,
				|r| &r.end,
			)
		)
	}

	/**
	* Load JSON from a file inside companyfacts.zip
	*/
	pub fn load_json_from_file(&mut self, file_name: &str) -> Result<Value, Box<dyn std::error::Error>>
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

		let assets: Vec<Assets> = self.extract_assets(&json_submission)?;

		let common_stock_shares_outstanding: Vec<
			CommonStockSharesOutstanding
		> = self.extract_common_stock_shares_outstanding(
			&json_submission
		)?;

		let entity_common_stock_shares_outstanding: Vec<
			EntityCommonStockSharesOutstanding
		> = self.extract_entity_common_stock_shares_outstanding(
			&json_submission
		)?;

		Ok(
			Companyfacts
			{
				assets,
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
