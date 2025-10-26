use diesel::mysql::MysqlConnection;

use crate::config::db::DbPool;

#[derive(Debug, Clone)]
pub struct Config {
    pub url: String,
    pub port: String,
    pub countries_api_url: String,
    pub exchange_rate_api_url: String,
    pub db: DbPool,
}

impl Config {
    pub fn init() -> Config {
        let port = std::env::var("PORT").expect("PORT must be set");
        let url = std::env::var("URL").expect("URL must be set");
        let countries_api_url = std::env::var("COUNTRIES_API_URL").expect("COUNTRIES_API_URL must be set");
        let exchange_rate_api_url = std::env::var("EXCHANGE_RATE_API_URL").expect("EXCHANGE_RATE_API_URL must be set");
        let db_url = DbPool::new();

        Config {
            port,
            url,
            countries_api_url,
            exchange_rate_api_url,
            db: db_url,
        }
    }
}

unsafe impl Send for Config {}
unsafe impl Sync for Config {}
