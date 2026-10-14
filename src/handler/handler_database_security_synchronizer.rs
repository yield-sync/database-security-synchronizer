use chrono::{ Local };

use std::collections::HashMap;
use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::database::table_security::TableSecurity;
use crate::handler::HandlerApiSec;
use crate::handler::UpdatedSecCompanyfactsAndSubmissions;
use crate::handler::HandlerFilingCommonStockSharesOutstanding;
use crate::handler::HandlerFilingEntityCommonStockSharesOutstanding;
use crate::handler::HandlerSecurityExchangeTicker;
use crate::handler::HandlerFiling;
use crate::handler::data::handler_filing_assets::HandlerFilingAssets;
use crate::handler::data::handler_sec_submission_file_hash::HandlerSecSubmissionFileHash;
use crate::handler::file::zip::HandlerFileCompanyfactsZip;
use crate::handler::file::zip::HandlerFileSubmissionsZip;
use crate::schema::Companyfacts;
use crate::schema::SubmissionsData;

use crate::{ log_debug, log_ultradebug, log_error, log_info, log_warn };
use crate::handler::{ HandlerSecurity, SynchronizeSecurity };


pub struct HandlerDatabaseSecuritySynchronizer
{
	h_api_sec: HandlerApiSec,
	h_filing_entity_common_stock_shares_outstanding: HandlerFilingEntityCommonStockSharesOutstanding,
	h_filing_assets: HandlerFilingAssets,
	h_filing_common_stock_shares_outstanding: HandlerFilingCommonStockSharesOutstanding,
	h_sec_submission_file_hash: HandlerSecSubmissionFileHash,
	h_security: HandlerSecurity,
	h_security_exchange_ticker: HandlerSecurityExchangeTicker,
	h_filing: HandlerFiling,
	t_security: TableSecurity,
}


impl HandlerDatabaseSecuritySynchronizer
{
	/**
	* Constructor
	*/
	pub async fn new(db_connection: Arc<DatabaseConnection>) -> Result<Self, Box<dyn std::error::Error>>
	{
		Ok(
			Self
			{
				h_api_sec: HandlerApiSec::new(),
				h_sec_submission_file_hash: HandlerSecSubmissionFileHash::new(db_connection.clone()),
				h_filing_entity_common_stock_shares_outstanding: HandlerFilingEntityCommonStockSharesOutstanding::new(
					db_connection.clone()
				),
				h_filing_assets: HandlerFilingAssets::new(db_connection.clone()),
				h_filing_common_stock_shares_outstanding: HandlerFilingCommonStockSharesOutstanding::new(
					db_connection.clone()
				),
				h_security: HandlerSecurity::new(db_connection.clone()),
				h_security_exchange_ticker: HandlerSecurityExchangeTicker::new(db_connection.clone()),
				h_filing: HandlerFiling::new(db_connection.clone()),
				t_security: TableSecurity::new(db_connection.clone()),
			}
		)
	}


	/**
	* @visibility: Private
	*/
	fn _normalize_phone(phone: &str) -> Option<String>
	{
		let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

		if digits.is_empty()
		{
			None
		}
		else
		{
			Some(digits)
		}
	}

	async fn _process_submission_file(
		&self,
		handler_file_companyfacts_zip: &mut HandlerFileCompanyfactsZip,
		handler_file_submissions_zip: &mut HandlerFileSubmissionsZip,
		submission_file_name: &str,
		submission_file_hash: &str,
		current: usize,
		total_files: usize,
	) -> Result<(), Box<dyn std::error::Error>>
	{
		let percent_complete: f64 = (current as f64 / total_files as f64) * 100.0;

		let submissions_data: SubmissionsData = handler_file_submissions_zip.extract_submissions_data(
			submission_file_name
		)?;

		if submissions_data.tickers.is_empty()
		{
			log_ultradebug!("No tickers found in submissions/{}, skipping..", submission_file_name);

			return Ok(());
		}

		log_info!("------------------------------------------------------------");
		log_info!("Synchronizing submissions/{}", submission_file_name);
		log_info!("{:.2}% Complete - ({}/{})", percent_complete, current, total_files);
		log_info!("CIK: {}", submissions_data.cik);
		log_info!("Name: {}", submissions_data.name);
		log_info!("Tickers: {}", submissions_data.tickers.join(", "));
		log_info!("Exchanges: {}", submissions_data.exchanges.join(", "));

		if submissions_data.exchanges.iter().any(|exchange| exchange.eq_ignore_ascii_case("OTC"))
		{
			log_info!("OTC exchange not supported for submissions/{}, skipping..", submission_file_name);

			return Ok(());
		}

		// Skip if the security exists and this exact submission file was already processed
		if self.t_security.get_by_cik(&submissions_data.cik).await?.is_some()
		{
			if self.h_sec_submission_file_hash.hash_exists(submission_file_name, submission_file_hash).await?
			{
				log_debug!("Hash found in table sec_submission_file_hash, synchronize NOT required");
				log_info!("[SKIP] Synchronize not required");

				return Ok(());
			}

			log_debug!("Hash NOT found in table sec_submission_file_hash, synchronize required");
		}

		log_info!("Synchronizing security now..");

		let synchronize_security = SynchronizeSecurity {
			cik: submissions_data.cik.clone(),
			business_country: submissions_data.business_country,
			business_city: submissions_data.business_city,
			business_state: submissions_data.business_state,
			business_street1: submissions_data.business_street1,
			business_zip: submissions_data.business_zip,
			description: submissions_data.description,
			ein: submissions_data.ein,
			entity_type: submissions_data.entity_type,
			name: submissions_data.name,
			phone: Self::_normalize_phone(&submissions_data.phone).unwrap_or_default(),
			sic: submissions_data.sic,
			website: submissions_data.website,
		};

		if let Err(e) = self.h_security.synchronize(&synchronize_security).await
		{
			log_error!("Failed to synchronize security: {}", e);
		}

		if let Err(e) = self.h_security_exchange_ticker.synchronize(
			&submissions_data.cik,
			&submissions_data.exchanges,
			&submissions_data.tickers,
		).await
		{
			log_error!("Failed to synchronize security_exchange_ticker: {}", e);
		}

		if let Err(e) = self.h_filing.synchronize(&submissions_data.cik, &submissions_data.filings).await
		{
			log_error!("Failed to synchronize filing with error: {}", e);
		}

		if handler_file_companyfacts_zip.file_exists(submission_file_name)
		{
			let companyfacts: Companyfacts = handler_file_companyfacts_zip.extract_data(submission_file_name)?;

			if let Err(e) = self.h_filing_assets.synchronize(&companyfacts.assets).await
			{
				log_error!("Failed to synchronize filing_assets: {}", e);
			}

			if let Err(e) = self.h_filing_common_stock_shares_outstanding.synchronize(
				&companyfacts.common_stock_shares_outstanding,
			).await
			{
				log_error!("Failed to synchronize filing_common_stock_shares_outstanding: {}", e);
			}

			if let Err(e) = self.h_filing_entity_common_stock_shares_outstanding.synchronize(
				&companyfacts.entity_common_stock_shares_outstanding,
			).await
			{
				log_error!("Failed to synchronize filing_entity_common_stock_shares_outstanding: {}", e);
			}
		}
		else
		{
			log_warn!("{} not found in companyfacts.zip.", submission_file_name);
		}

		self.h_sec_submission_file_hash.synchronize(
			&submission_file_name.to_string(),
			&submission_file_hash.to_string()
		).await?;

		Ok(())
	}

	async fn _process_submission_files(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		let UpdatedSecCompanyfactsAndSubmissions
		{
			mut handler_file_companyfacts_zip,
			mut handler_file_submissions_zip,
		} = self.h_api_sec.get_updated_companyfacts_and_submissions().await?;

		let file_names_to_hashes: HashMap<String, String> = handler_file_submissions_zip.compute_file_names_to_hashes()?;

		let total_files: usize = file_names_to_hashes.len();

		log_info!("Processing {} files..", total_files);

		let mut failed_files: usize = 0;

		for (index, (submission_file_name, submission_file_hash)) in file_names_to_hashes.into_iter().enumerate()
		{
			if let Err(e) = self._process_submission_file(
				&mut handler_file_companyfacts_zip,
				&mut handler_file_submissions_zip,
				&submission_file_name,
				&submission_file_hash,
				index + 1,
				total_files,
			).await
			{
				failed_files += 1;

				log_error!("Failed to process submissions/{}: {}", submission_file_name, e);
			}
		}

		if failed_files > 0
		{
			log_warn!("{} of {} files failed to process", failed_files, total_files);
		}

		Ok(())
	}

	/**
	* @visibility: Public
	*/
	pub async fn synchronize(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!("Building security profile at {}", Local::now().format("%Y-%m-%d %H:%M:%S"));

		self._process_submission_files().await?;

		log_info!("Security profiles built successfully");

		Ok(())
	}
}
