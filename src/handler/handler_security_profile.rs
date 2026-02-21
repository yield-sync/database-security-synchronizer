use chrono::{ Local };

use std::sync::Arc;

use crate::database::table_security::TableSecurity;
use crate::database::database_connection::DatabaseConnection;
use crate::handler::HandlerApiSec;
use crate::handler::HandlerCacheSubmissionsFileWithNoTickers;
use crate::handler::HandlerSecurityExchangeTicker;
use crate::handler::HandlerSecurityFilingCommonStockSharesOutstanding;
use crate::handler::HandlerSecurityFiling;
use crate::handler::UpdatedSecCompanyfactsAndSubmissions;
use crate::schema::SubmissionsData;
use crate::schema::Companyfacts;

use crate::{ log_debug, log_error, log_info, log_warn };
use crate::handler::{ HandlerSecurity, SynchronizeSecurity };


pub struct HandlerSecurityProfile
{}


impl HandlerSecurityProfile
{
	/**
	* @visibility: Public
	*/
	pub fn new() -> Self
	{
		Self
		{}
	}


	/**
	* Run Creating Security Profile tasks
	*/
	pub async fn synchronize(&self) -> Result<(), Box<dyn std::error::Error>>
	{
		log_info!("Building security profile at {}", Local::now().format("%Y-%m-%d %H:%M:%S"));

		let handler_api_sec = HandlerApiSec::new();

		let mut handler_cache_submissions_file_with_no_tickers = HandlerCacheSubmissionsFileWithNoTickers::new();

		let db_connection = Arc::new(DatabaseConnection::new().await?);

		let t_security = TableSecurity::new(db_connection.clone());

		let UpdatedSecCompanyfactsAndSubmissions
		{
			mut handler_companyfacts_zip,
			mut handler_previous_submissions_zip,
			mut submissions_zip_handler,
		} = handler_api_sec.get_updated_companyfacts_and_submissions().await?;

		let submissions_file_names_to_hashs = submissions_zip_handler.compute_file_names_to_hashes()?;

		let previous_submissions_file_names_to_hashes = if let Some(
			handler_previous_submissions_zip
		) = &mut handler_previous_submissions_zip
		{
			Some(handler_previous_submissions_zip.compute_file_names_to_hashes()?)
		}
		else
		{
			None
		};

		for (s_file_name, s_hash) in submissions_file_names_to_hashs
		{
			log_debug!("Processing submissions/{}", s_file_name);

			if handler_cache_submissions_file_with_no_tickers.is_tickerless_submission_file(&s_file_name)
			{
				log_debug!(
					"[SKIP] cache.submission-file-with-no-tickers contains submissions/{}, skipping..",
					s_file_name
				);

				continue;
			}

			let submissions_data: SubmissionsData = submissions_zip_handler.extract_submissions_data(&s_file_name)?;

			if submissions_data.tickers.is_empty()
			{
				log_debug!(
					"[SKIP] {} has invalid 'tickers'. adding to cache.json 'submission-file-with-no-tickers'..",
					s_file_name
				);

				handler_cache_submissions_file_with_no_tickers.add_tickerless_submission_file_name(&s_file_name);

				continue;
			}

			log_info!("Synchronizing submissions/{}", s_file_name);
			log_info!("CIK: {}", submissions_data.cik);
			log_info!("Name: {}", submissions_data.name);
			log_info!("Tickers: {}", submissions_data.tickers.join(", "));
			log_info!("Exchanges: {}", submissions_data.exchanges.join(", "));

			let mut synchronize_required: bool = false;

			// Search database for security with cik
			if let Some(_) = t_security.get_by_cik(&submissions_data.cik).await?
			{
				if let Some(previous_submissions_file_names_to_hashes) = &previous_submissions_file_names_to_hashes
				{
					if let Some(previous_submission_json_hash) = previous_submissions_file_names_to_hashes.get(
						&s_file_name
					)
					{
						if previous_submission_json_hash != &s_hash
						{
							synchronize_required = true;
						}
					}
				}
			}
			else
			{
				synchronize_required = true;
			}

			if !synchronize_required
			{
				log_info!("[SKIP] Synchronize not required. Skipping..");

				continue;
			}

			log_info!("Synchronize required..");

			(HandlerSecurity::new(db_connection.clone())).synchronize(
				&SynchronizeSecurity {
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
				},
			).await?;

			if let Err(e) = HandlerSecurityExchangeTicker::new(db_connection.clone()).synchronize(
				&submissions_data.cik,
				&submissions_data.tickers,
				&submissions_data.exchanges,
			).await
			{
				log_error!("Failed to synchronize tickers and exchanges: {}", e);
			}

			(HandlerSecurityFiling::new(db_connection.clone())).synchronize(
				&submissions_data.cik,
				&submissions_data.filings
			).await?;

			let companyfacts: Option<Companyfacts> = if handler_companyfacts_zip.file_exists(&s_file_name)
			{
				Some(handler_companyfacts_zip.extract_data(&s_file_name)?)
			}
			else
			{
				log_warn!("{} not found in companyfacts.zip file not found. Skipping..", &s_file_name);

				None
			};

			if let Some(companyfacts) = companyfacts
			{
				(HandlerSecurityFilingCommonStockSharesOutstanding::new(db_connection.clone())).synchronize(
					&companyfacts.common_stock_shares_outstanding,
				).await?;
			}
		}

		db_connection.close().await?;

		log_info!("Security profiles built successfully.");

		Ok(())
	}
}
