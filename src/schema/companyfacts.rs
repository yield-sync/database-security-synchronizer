#[derive(Debug)]
pub struct CompanyfactsCommonStockSharesOutstanding
{
	pub security_filing_accession_number: String,
	pub end: String,
	pub filed: String,
	pub fp: String,
	pub fy: i64,
	pub form: String,
	pub common_stock_shares_outstanding: i64,
}

#[derive(Debug)]
pub struct Companyfacts
{
	pub common_stock_shares_outstanding: Vec<CompanyfactsCommonStockSharesOutstanding>,
}
