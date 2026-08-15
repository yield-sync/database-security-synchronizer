use chrono::{ Local };

use std::sync::Arc;

use crate::database::database_connection::DatabaseConnection;
use crate::database::table_security::TableSecurity;
use crate::handler::HandlerApiSec;
use crate::handler::UpdatedSecCompanyfactsAndSubmissions;
use crate::handler::HandlerFilingCommonStockSharesOutstanding;
use crate::handler::HandlerFilingEntityCommonStockSharesOutstanding;
use crate::handler::HandlerSecurityExchangeTicker;
use crate::handler::HandlerSecurityFiling;
use crate::handler::data::handler_filing_assets::HandlerFilingAssets;
use crate::handler::data::handler_sec_submission_file_hash::HandlerSecSubmissionFileHash;
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
	h_security_filing: HandlerSecurityFiling,
	t_security: TableSecurity
}


impl HandlerDatabaseSecuritySynchronizer
{
	/**
	* @visibility: Public
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
				h_security_filing: HandlerSecurityFiling::new(db_connection.clone()),
				t_security: TableSecurity::new(db_connection.clone()),
			}
		)
	}


	/**
	* Run Creating Security Profile tasks
	*/
	pub async fn synchronize(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!("Building security profile at {}", Local::now().format("%Y-%m-%d %H:%M:%S"));

		let UpdatedSecCompanyfactsAndSubmissions
		{
			mut handler_file_companyfacts_zip,
			mut handler_file_submissions_zip,
		} = self.h_api_sec.get_updated_companyfacts_and_submissions().await?;

		for (submission_file_name, submission_file_hash) in handler_file_submissions_zip.compute_file_names_to_hashes()?
		{
			log_ultradebug!("Processing file submissions/{}", submission_file_name);

			let submissions_data: SubmissionsData = match handler_file_submissions_zip.extract_submissions_data(
				&submission_file_name
			)
			{
				Ok(data) => data,
				Err(e) =>
				{
					log_error!("Failed to extract submissions data: {}", e);

					continue;
				}
			};

			if submissions_data.tickers.is_empty()
			{
				log_ultradebug!("No tickers found in submissions/{}, skipping..", submission_file_name);

				continue;
			}

			log_info!("------------------------------------------------------------");
			log_info!("Synchronizing submissions/{}", submission_file_name);
			log_info!("CIK: {}", submissions_data.cik);
			log_info!("Name: {}", submissions_data.name);
			log_info!("Tickers: {}", submissions_data.tickers.join(", "));
			log_info!("Exchanges: {}", submissions_data.exchanges.join(", "));

			let mut synchronize_required: bool = true;

			// Search database for security with cik
			if let Some(_) = self.t_security.get_by_cik(&submissions_data.cik).await?
			{
				if self.h_sec_submission_file_hash.hash_exists(&submission_file_name, &submission_file_hash).await?
				{
					log_debug!("Hash found in table sec_submission_file_hash, synchronize NOT required");

					synchronize_required = false;
				}
				else
				{
					log_debug!("Hash NOT found in table sec_submission_file_hash, synchronize required");
				}
			}

			if !synchronize_required
			{
				log_info!("[SKIP] Synchronize not required. Skipping");

				continue;
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
				phone: submissions_data.phone,
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

			if let Err(e) = self.h_security_filing.synchronize(
				&submissions_data.cik,
				&submissions_data.filings
			).await
			{
				log_error!("Failed to synchronize security_filing with error: {}", e);
			}

			if handler_file_companyfacts_zip.file_exists(&submission_file_hash)
			{
				let companyfacts: Companyfacts = handler_file_companyfacts_zip.extract_data(&submission_file_hash)?;

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
				log_warn!("{} not found in companyfacts.zip.", &submission_file_name);
			}

			if let Err(e) = self.h_sec_submission_file_hash.synchronize(
				&submission_file_name.to_string(),
				&submission_file_hash.to_string()
			).await
			{
				log_error!("Failed to synchronize sec_submission_file_hash: {}", e);
			}
		}

		log_info!("Security profiles built successfully");

		Ok(())
	}
}
