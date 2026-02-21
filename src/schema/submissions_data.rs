use chrono::{ NaiveDate, NaiveDateTime };


#[derive(Debug)]
pub struct SubmissionsDataFilings
{
	pub accession_number: String,
	pub filing_date: NaiveDate,
	pub form: String,
	pub report_date: Option<NaiveDate>,
	pub acceptance: NaiveDateTime,
}

#[derive(Debug)]
pub struct SubmissionsData
{
	pub cik: String,

	pub tickers: Vec<String>,
	pub exchanges: Vec<String>,

	pub business_country: String,
	pub business_city: String,
	pub business_state: String,
	pub business_street1: String,
	pub business_zip: String,
	pub description: String,
	pub ein: String,
	pub entity_type: String,
	pub name: String,
	pub phone: String,
	pub sic: String,
	pub website: String,

	pub filings: Vec<SubmissionsDataFilings>,
}
