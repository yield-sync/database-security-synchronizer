use chrono::{ Local };

use std::sync::Arc;

use crate::database::table_security::TableSecurity;
use crate::database::database_connection::DatabaseConnection;
use crate::handler::HandlerApiSec;
use crate::handler::HandlerSecurityExchangeTicker;
use crate::handler::HandlerSecurityFilingCommonStockSharesOutstanding;
use crate::handler::HandlerSecurityFiling;
use crate::handler::UpdatedSecCompanyfactsAndSubmissions;
use crate::handler::data::handler_sec_submission_file_hash::HandlerSecSubmissionFileHash;
use crate::schema::SubmissionsData;
use crate::schema::Companyfacts;

use crate::{ log_debug, log_error, log_info, log_superdebug, log_warn };
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

		let db_connection = Arc::new(DatabaseConnection::new().await?);

		let t_security = TableSecurity::new(db_connection.clone());

		let handler_sec_submission_file_hash = HandlerSecSubmissionFileHash::new(db_connection.clone());

		let UpdatedSecCompanyfactsAndSubmissions
		{
			mut handler_companyfacts_zip,
			mut handler_submissions_zip,
		} = handler_api_sec.get_updated_companyfacts_and_submissions().await?;

		let submissions_file_names_to_hashs = handler_submissions_zip.compute_file_names_to_hashes()?;

		for (s_file_name, s_hash) in submissions_file_names_to_hashs
		{
			log_superdebug!("Processing submissions/{}", s_file_name);

			let submissions_data: SubmissionsData = handler_submissions_zip.extract_submissions_data(&s_file_name)?;

			if submissions_data.tickers.is_empty()
			{
				log_superdebug!(
					"No tickers found in submissions/{}, skipping..",
					s_file_name
				);

				continue;
			}

			log_info!("📣 Synchronizing submissions/{}", s_file_name);
			log_info!("CIK: {}", submissions_data.cik);
			log_info!("Name: {}", submissions_data.name);
			log_info!("Tickers: {}", submissions_data.tickers.join(", "));
			log_info!("Exchanges: {}", submissions_data.exchanges.join(", "));

			let mut synchronize_required: bool = false;

			// Search database for security with cik
			if let Some(_) = t_security.get_by_cik(&submissions_data.cik).await?
			{
				if !handler_sec_submission_file_hash.hash_exists(&s_file_name, &s_hash).await?
				{
					log_debug!("Hash not found in database..");

					synchronize_required = true;
				}
				else
				{
					log_debug!("Hash found in database..");
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
				&submissions_data.exchanges,
				&submissions_data.tickers,
			).await
			{
				log_error!("Failed to synchronize security_exchange_ticker with error: {}", e);
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

			handler_sec_submission_file_hash.synchronize(&s_file_name.to_string(), &s_hash.to_string()).await?;
		}

		db_connection.close().await?;

		log_info!("Security profiles built successfully.");

		Ok(())
	}
}
