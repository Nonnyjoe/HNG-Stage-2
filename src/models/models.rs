use chrono::NaiveDateTime;
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use crate::schema::countries;
use diesel::prelude::*;
use crate::schema::cache_metadata;

// ─────────────────────────────
//  Queryable + Selectable struct
// ─────────────────────────────
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = countries)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Country {
    pub id: i32,
    pub name: Option<String>,
    pub capital: Option<String>,
    pub region: Option<String>,
    pub population: Option<i64>,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<f64>,
    pub estimated_gdp: Option<f64>,
    pub flag_url: Option<String>,
    pub last_refreshed_at: Option<NaiveDateTime>,
}

// ─────────────────────────────
//  Insertable + AsChangeset struct
// ─────────────────────────────
#[derive(Insertable, AsChangeset, Debug, Deserialize)]
#[diesel(table_name = countries)]
pub struct UpsertCountry<'a> {
    pub name: Option<&'a str>,
    pub capital: Option<&'a str>,
    pub region: Option<&'a str>,
    pub population: Option<i64>,
    pub currency_code: Option<&'a str>,
    pub exchange_rate: Option<f64>,
    pub estimated_gdp: Option<f64>,
    pub flag_url: Option<&'a str>,
    pub last_refreshed_at: Option<NaiveDateTime>,
}


#[derive(Insertable, Queryable, Serialize, Debug)]
#[diesel(table_name = cache_metadata)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct CacheMetadata {
    pub file_path: String,
    pub total_countries: i32,
    pub top_countries_json: String, // serialized JSON array
    pub last_refreshed_at: NaiveDateTime,
}