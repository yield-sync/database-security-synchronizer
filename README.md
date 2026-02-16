1. Upon running the application, the code will check that the zip files are older than 24 hours. If so, it should than rename `companyfacts.zip` to `old.companyfacts.zip` and download the new one using the `handler_api_sec.rs`

*Please avoid extracting the zips to the physical drive as this will take up a lot of disk space*

2. After downloading the zips, the handler `companyfacts_handler.rs` should have a function called `process_companyfacts` that opens the `companyfacts.zip` file and `old.companyfacts.zip` and compare each file to see if they have updated or not. If they have, it should update the database with the new information.

3. It should also do the same for `submissions.zip`
