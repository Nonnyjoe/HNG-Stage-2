use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use diesel::SelectableHelper;
use crate::models::countries_models::Country;
use crate::models::models::CacheMetadata;
use crate::schema::cache_metadata::top_countries_json;
use crate::schema::countries::dsl::*;
use crate::schema::cache_metadata::dsl::*;
use crate::models::models::Country as CountryModel;

#[derive(Debug, Clone)]
pub struct DbPool {
    pub db_url: String,
}

pub fn establish_connection(database_url: String) -> MysqlConnection {
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|e| panic!("Error connecting to {}: {e}", database_url))
}

impl DbPool {
    pub fn new() -> Self {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
        DbPool { db_url: database_url }
    }

    pub fn insert_or_update_country(&self, country: Country) -> QueryResult<usize> {

       let mut conn =  establish_connection(self.db_url.clone());

        let exists = countries
            .filter(name.eq(&country.name))
            .limit(1)
            .select(CountryModel::as_select())
            .load(&mut conn)
            .expect("Error loading country");

        let upsert_data = country.struct_to_upsert_country();

        println!("Upserting country: {:?}", upsert_data);

        if !exists.is_empty() {
            diesel::update(countries.filter(name.eq(&country.name)))
                .set(&upsert_data)
                .execute(&mut conn)
        } else {
            diesel::insert_into(countries)
                .values(&upsert_data)
                .execute(&mut conn)
        }
    }

    pub fn get_all_countries(&self) -> QueryResult<Vec<CountryModel>> {
        let mut conn = establish_connection(self.db_url.clone());
        countries.load::<CountryModel>(&mut conn)
    }

    pub fn get_country_by_name(&self, country_name: &str) -> QueryResult<Option<CountryModel>> {
        let mut conn = establish_connection(self.db_url.clone());
        countries
            .filter(name.eq(country_name))
            .first::<CountryModel>(&mut conn)
            .optional()
    }

    pub fn delete_country_by_name(&self, country_name: &str) -> QueryResult<usize> {
        let mut conn = establish_connection(self.db_url.clone());
        diesel::delete(countries.filter(name.eq(country_name))).execute(&mut conn)
    }

    pub fn save_summary_metadata(&self, top_countries: Vec<CountryModel>, last_refreshed_at_: Option<String>) -> QueryResult<usize> {
        let mut conn = establish_connection(self.db_url.clone());

        let top_json = serde_json::json!(top_countries).to_string();

        let new_metadata = CacheMetadata {
            file_path: "cache/summary.png".to_string(),
            total_countries: top_countries.len() as i32,
            top_countries_json: top_json,
            last_refreshed_at: last_refreshed_at_.as_ref().and_then(|ts| {
            chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.fZ").ok()
        }).expect("Invalid timestamp format"),
        };

        diesel::insert_into(cache_metadata)
            .values(&new_metadata)
            .execute(&mut conn)
    }
}