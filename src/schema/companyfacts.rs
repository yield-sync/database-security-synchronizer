#[derive(Debug)]
pub struct Assets
{
	pub security_filing_accession_number: String,
	pub end: String,
	pub fp: String,
	pub fy: i64,
	pub val: i64,
}

#[derive(Debug)]
pub struct CommonStockSharesOutstanding
{
	pub security_filing_accession_number: String,
	pub end: String,
	pub fp: String,
	pub fy: i64,
	pub val: i64,
}

#[derive(Debug)]
pub struct EntityCommonStockSharesOutstanding
{
	pub security_filing_accession_number: String,
	pub end: String,
	pub fp: String,
	pub fy: i64,
	pub val: i64,
}

#[derive(Debug)]
pub struct Companyfacts
{
	pub assets: Vec<Assets>,
	pub common_stock_shares_outstanding: Vec<CommonStockSharesOutstanding>,
	pub entity_common_stock_shares_outstanding: Vec<EntityCommonStockSharesOutstanding>,
}
