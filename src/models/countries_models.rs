use crate::{models::models::UpsertCountry, schema::countries::currency_code};

#[derive(Debug, Clone)]
pub struct Country {
    pub name: Option<String>,
    pub capital: Option<String>,
    pub region: Option<String>,
    pub population: Option<u64>,
    pub currencies: Option<Currency>,
    pub independent: Option<bool>,
    pub exchange_rate: Option<f64>,
    pub estimated_gdp: Option<f64>,
    pub flag_url: Option<String>,
    pub last_refreshed_at: Option<String>,
}


#[derive(Debug, Clone)]
pub struct Currency {
    pub code: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
}


impl Country {
    pub fn new(
        name: String,
        capital: String,
        region: String,
        population: u64,
        flag_url: String,
        currencies: Currency,
        independent: Option<bool>
    ) -> Self {
        Country {
            name: Some(name),
            capital: Some(capital),
            region: Some(region),
            population: Some(population),
            flag_url: Some(flag_url),
            currencies: Some(currencies),
            independent,
            last_refreshed_at: None,
            exchange_rate: None,
            estimated_gdp: None,
        }
    }

    pub fn new_from_json(json: &serde_json::Value) -> Self {
        let name = json.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
        let capital = json.get("capital").and_then(|v| v.as_str()).map(|s| s.to_string());
        let region = json.get("region").and_then(|v| v.as_str()).map(|s| s.to_string());
        let population = json.get("population").and_then(|v| v.as_u64());
        let flag_url = json.get("flag").and_then(|v| v.as_str()).map(|s| s.to_string());

        // let currencies_json = json.get("currencies")?.as_array()?;
        let currencies = json
            .get("currencies")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.get(0))
            .map(|c| Currency {
                code: c.get("code").and_then(|v| v.as_str()).map(|s| s.to_string()),
                name: c.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                symbol: c.get("symbol").and_then(|v| v.as_str()).map(|s| s.to_string()),
            });

        let independent = json.get("independent").and_then(|v| v.as_bool());

       Country {
            name,
            capital,
            region,
            population,
            flag_url,
            currencies: currencies,
            independent,
            last_refreshed_at: None,
            exchange_rate: None,
            estimated_gdp: None,
        }
    }

    pub fn set_exchange_rate(&mut self, rate: f64) {
        self.exchange_rate = Some(rate);
    }

    pub fn set_estimated_gdp(&mut self, gdp: f64) {
        self.estimated_gdp = Some(gdp);
    }

    pub fn set_last_refreshed_at(&mut self, timestamp: String) {
        self.last_refreshed_at = Some(timestamp);
    }

    pub fn get_currency_code(&self) -> Option<&str> {
        let country = self.clone();
        if country.currencies.is_none() {
            return None;
        } else {
            if country.clone().currencies.unwrap().code.is_none() {
                return None;
            } else {
                return self.currencies.as_ref()?.code.as_deref()
            }
        }
    }

    pub fn get_currency_code_owned(&self) -> Option<String> {
        let country = self.clone();
        if country.currencies.is_none() {
            return None;
        } else {
            if country.clone().currencies.unwrap().code.is_none() {
                return None;
            } else {
                return self.currencies.as_ref()?.code.clone()
            }
        }
    }

    pub fn structure_country_for_return(&self, index: usize) -> serde_json::Value {
        serde_json::json!({
            "id": index,
            "name": self.name,
            "capital": self.capital,
            "region": self.region,
            "population": self.population,
            "currency_code": self.currencies.as_ref().and_then(|c| c.code.clone()),
            "exchange_rate": self.exchange_rate,
            "estimated_gdp": self.estimated_gdp,
            "flag_url": self.flag_url,
            "last_refreshed_at": self.last_refreshed_at,
        })
    }

pub fn struct_to_upsert_country(&self) -> UpsertCountry {
    let population = self.population.map(|p| p as i64);

    UpsertCountry {
        name: self.name.as_deref(),
        capital: self.capital.as_deref(),
        region: self.region.as_deref(),
        population,
        currency_code: self.get_currency_code(),
        exchange_rate: self.exchange_rate,
        estimated_gdp: self.estimated_gdp,
        flag_url: self.flag_url.as_deref(),
        last_refreshed_at: self.last_refreshed_at.as_ref().and_then(|ts| {
            chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.fZ").ok()
        }),
    }
}
}