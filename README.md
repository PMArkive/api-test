# api-test

Test suite for demos.tf api 

## Usage

Start the test with the following environment variables

 - `DB_URL` - [sqlx](https://github.com/launchbadge/sqlx) database url for the database used by the api
 - `BASE_URL` - base url for the api
 - `EDIT_KEY` - edit key for the api
 
Note that the test suite is destructive, all data saved in the database will be wiped for each test run.