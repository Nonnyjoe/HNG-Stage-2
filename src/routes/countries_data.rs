use crate::{AppState, models::countries_models::{Country, Currency}};
use actix_web::{HttpResponse, Responder, get, post, delete, web, HttpRequest, http::header::ContentType};
use reqwest::Client;
use rand::Rng;
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_text_mut};
use rusttype::{Font, Scale};
use chrono::Local;
use std::{fs, io::Error, path::PathBuf};
use std::path::Path;
use ab_glyph::{FontArc, PxScale};
use actix_files::NamedFile;

#[derive(serde::Deserialize, Debug, Clone)]
enum SearchFilter {
    region(String),
    currency(String),
    sort(SortFilter),
}

#[derive(serde::Deserialize, Debug, Clone)]
enum SortFilter {
    gdp_desc,
}


#[derive(serde::Deserialize, Debug)]
struct SearchQuery {
    region: Option<String>,
    currency: Option<String>,
    sort: Option<SortFilter>,
}


#[post("/countries/refresh")]
async fn refresh_countries_data(_data: web::Data<AppState>) -> impl Responder {
    let countries_url = &_data.env.countries_api_url;
    let exchange_rate_url = &_data.env.exchange_rate_api_url;
    let last_refreshed_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let db = &_data.env.db;
    let mut countries_4_image = Vec::new();

    println!("Countries API URL: {}", countries_url);
    println!("Exchange Rate API URL: {}", exchange_rate_url);

    match get_exchange_rates(exchange_rate_url).await {
        Ok(rates) => {
            match fetch_countries_data(countries_url).await {
                Ok(countries) => {
                    println!("Fetched {} countries", countries.len());
                    countries_4_image = countries.clone();

                    for mut country in countries.into_iter() {
                        let code = country.get_currency_code_owned();
                        if let Some(c_code) = code {
                            match rates.get(&c_code) {
                                Some(rate) => {
                                    let random_no = rand::rng().random_range(1000..=2000);
                                    println!("Exchange rate for {}: {}, with random no: {}", c_code, rate, random_no);
                                    country.set_exchange_rate(rate.as_f64().unwrap_or(0.0));
                                    let estimated_gdp = (country.population.unwrap_or(0) as f64 * random_no as f64) / country.exchange_rate.unwrap_or(0.0);
                                    country.set_estimated_gdp(estimated_gdp);
                                },
                                None => {
                                    country.estimated_gdp = None;
                                    country.exchange_rate = None;
                                    println!("No exchange rate found for currency code: {}", c_code);
                                }
                            }
                        } else {
                            country.estimated_gdp = Some(0.0);
                            country.exchange_rate = Some(0.0);
                            println!("No currency code found for country: {}", country.clone().name.unwrap_or("Unknown".to_string()));
                        }
                        
                        country.set_last_refreshed_at(last_refreshed_at.clone());
                        println!("Country: {:?}", country);
                        db.insert_or_update_country(country).unwrap();
                    }
                },
                Err(e) => {
                    println!("Error fetching countries data: {}", e);
                    let json_response = serde_json::json!({
                        "error": "External data source unavailable",
                        "details": format!("Could not fetch data from {}", countries_url),
                    });
                    return HttpResponse::ServiceUnavailable().json(json_response);
                }
            }
        },
        Err(e) => {
            println!("Error fetching initial exchange rates: {}", e);
            let json_response = serde_json::json!({
                "error": "External data source unavailable",
                "details": format!("Could not fetch data from {}", exchange_rate_url),
            });
            return HttpResponse::ServiceUnavailable().json(json_response);
        }
    }

    match generate_summary_image(countries_4_image, last_refreshed_at) {
        Ok(_) => println!("Summary image generated successfully"),
        Err(e) => println!("Error generating summary image: {}", e),
    }
    HttpResponse::Ok().json(" Database Updated Succesfully" )
}


async fn fetch_countries_data(country_url: &str) -> Result<Vec<Country>, String> {
    let client = Client::new();

    let mut countries_response: Vec<Country> = Vec::new();
    let mut countries_error: String = String::new();

    match client.get(country_url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => {
                // println!("Countries JSON response: {:?}", json);
                for country in json.as_array().unwrap() {
                    let country_data = Country::new_from_json(country);
                    countries_response.push(country_data);
                }
            }
            Err(_) =>{ println!("Failed to parse Countries JSON response"); countries_error = "Failed to parse Countries JSON response".to_string(); return Err(countries_error) }
        },
        Err(e) =>{ println!("Failed to fetch countries data {:?}", e); countries_error = "Failed to fetch countries data".to_string() ; return Err(countries_error) }
    };
    Ok(countries_response)
}

async fn get_exchange_rates(exchange_rate_url: &str) -> Result<serde_json::Value, String> {
    let client = Client::new();

    let mut exchange_rate_error: String = String::new();

    match client.get(exchange_rate_url).send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => {
                match json.get("rates") {
                    Some(rates) => {
                        return Ok(rates.clone());
                    },
                    None => {
                        println!("No 'rates' field found in exchange rate response");
                        return Err("No 'rates' field found in exchange rate response".to_string());
                    }
                }
            }
            Err(_) =>{ println!("Failed to parse Exchange Rate JSON response"); exchange_rate_error = "Failed to parse Exchange Rate JSON response".to_string(); return Err(exchange_rate_error) }
        },
        Err(e) =>{ println!("Failed to fetch exchange rate data {:?}", e); exchange_rate_error = "Failed to fetch exchange rate data".to_string(); return Err(exchange_rate_error) }
    };
}

#[get("/countries")]
async fn get_countries_data(_data: web::Data<AppState>, query: web::Query<SearchQuery>) -> impl Responder {
    let q = query.into_inner();
    println!("Search Query: {:?}", q);

    let selected_filters: Vec<SearchFilter> = extract_filters_from_query(&q);
    let filtered_countries = apply_filters(_data, selected_filters.clone());

    match filtered_countries {
        Ok(mut countries) => {
            let mut return_countries = serde_json::json!([]);
            for (index, country) in countries.iter_mut().enumerate() {
                let country_json = country.structure_country_for_return(index + 1);
                return_countries.as_array_mut().unwrap().push(country_json);
            }
            return HttpResponse::Ok().json(return_countries);
        },
        Err(e) => {
            let reason = serde_json::from_str(&e).unwrap_or(serde_json::json!({ "unknown": "error" }));
            let json_response = serde_json::json!({
                "error": "Validation failed",
                "details": reason,
            });
            return HttpResponse::BadRequest().json(json_response);
        }
    }
    
}

fn apply_filters(_data: web::Data<AppState>, filters: Vec<SearchFilter>) -> Result<Vec<Country>, String> {
    println!("Applying filters: {:?}", filters);
    match get_all_countries(_data) {
        Ok(countries) => {
            let mut filtered_countries = countries;

            for filter in filters.into_iter() {
                match filter {
                    SearchFilter::region(region_name) => {
                        filtered_countries = filtered_countries.into_iter()
                            .filter(|c| c.region.as_ref().map_or(false, |r| r.to_lowercase() == region_name.to_lowercase()))
                            .collect();
                        if filtered_countries.is_empty() {
                            let details = serde_json::json!({
                                "region": "is required"
                            });
                            return Err(format!("{}", details));
                        }
                    },
                    SearchFilter::currency(currency_code) => {
                        filtered_countries = filtered_countries.into_iter()
                            .filter(|c| {
                                if let Some(curr) = &c.currencies {
                                    if let Some(code) = &curr.code {
                                        return code == &currency_code;
                                    }
                                }
                                false
                            })
                            .collect();
                        if filtered_countries.is_empty() {
                            let details = serde_json::json!({
                                "currency_code": "is required"
                            });
                            return Err(format!("{}", details));
                        }
                    },
                    SearchFilter::sort(sort_filter) => {
                        match sort_filter {
                            SortFilter::gdp_desc => {
                                filtered_countries.sort_by(|a, b| b.estimated_gdp.partial_cmp(&a.estimated_gdp).unwrap());
                            },
                        }
                    },
                }
            }

            return Ok(filtered_countries);
        },
        Err(e) => {
            println!("Error fetching all countries: {}", e);
            return Err("Error fetching all countries from db".to_string());
        }
    }
}

fn get_all_countries(_data: web::Data<AppState>) -> Result<Vec<Country>, String> {
    match _data.env.db.get_all_countries() {
        Ok(countries) => {
            if countries.is_empty() {
                return Err("No countries found in database".to_string());
            }
            // println!("countries fetched from db are: {:?}", countries);
            let countries_vec = countries.into_iter().map(|c| {
                Country {
                    name: c.name,
                    capital: c.capital,
                    region: c.region,
                    population: c.population.map(|p| p as u64),
                    flag_url: c.flag_url,
                    currencies: Some(Currency {
                        code: c.currency_code,
                        name: None,
                        symbol: None,
                    }),
                    independent: None,
                    last_refreshed_at: c.last_refreshed_at.map(|dt| dt.to_string()),
                    exchange_rate: c.exchange_rate,
                    estimated_gdp: c.estimated_gdp,
                }
            }).collect();
            return Ok(countries_vec);
        },
        Err(e) => {
            println!("Error retrieving countries from database: {}", e);
            return Err("Error retrieving countries from database".to_string());
        }
    }
}



fn extract_filters_from_query(query: &SearchQuery) -> Vec<SearchFilter> {
    let mut filters = Vec::new();

    if let Some(region) = &query.region {
        filters.push(SearchFilter::region(region.clone()));
    }

    if let Some(currency) = &query.currency {
        filters.push(SearchFilter::currency(currency.clone()));
    }

    if let Some(sort) = &query.sort {
        filters.push(SearchFilter::sort(sort.clone()));
    }

    filters
}


#[get("/countries/{name}")]
async fn get_country_by_name(_data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let input_value: String = path.into_inner();
    println!("Received get input for details: {}", input_value);

    if input_value.trim().is_empty() {
        let json_response = serde_json::json!({
            "error": "Validation failed",
            "details": {
                "name": "is required"
            }
        });
        return HttpResponse::BadRequest().json(json_response);
    }

    let db = &_data.env.db;
    match db.get_country_by_name(&input_value) {
        Ok(country_opt) => {
            if let Some(c) = country_opt {
                let country_data =  Country {
                    name: c.name,
                    capital: c.capital,
                    region: c.region,
                    population: c.population.map(|p| p as u64),
                    flag_url: c.flag_url,
                    currencies: Some(Currency {
                        code: c.currency_code,
                        name: None,
                        symbol: None,
                    }),
                    independent: None,
                    last_refreshed_at: c.last_refreshed_at.map(|dt| dt.to_string()),
                    exchange_rate: c.exchange_rate,
                    estimated_gdp: c.estimated_gdp,
                };
                let country_json = country_data.structure_country_for_return(1);
                return HttpResponse::Ok().json(country_json);
            } else {
                let json_response = serde_json::json!({
                    "error": "Country not found",
                    "details": format!("No country found with name: {}", input_value)
                });
                return HttpResponse::NotFound().json(json_response);
            }
        },
        Err(e) => {
            println!("Error retrieving country from database: {}", e);
            let json_response = serde_json::json!({
                "error": "Internal Server Error",
                "details": "Error retrieving country from database"
            });
            return HttpResponse::InternalServerError().json(json_response);
        }
    }
}



#[delete("/countries/{name}")]
async fn delete_country_data(_data: web::Data<AppState>, path: web::Path<String>) -> impl Responder {
    let input_value: String = path.into_inner();
    println!("Received Delete input for details: {}", input_value);
    if input_value.trim().is_empty() {
        let json_response = serde_json::json!({
            "error": "Validation failed",
            "details": {
                "name": "is required"
            }
        });
        return HttpResponse::BadRequest().json(json_response);
    }

    let db = &_data.env.db;

    match db.delete_country_by_name(&input_value) {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                let json_response = serde_json::json!({
                    "status": "success",
                    "message": format!("Country '{}' deleted successfully", input_value)
                });
                return HttpResponse::Ok().json(json_response);
            } else {
                let json_response = serde_json::json!({
                    "error": "Country not found",
                    "details": format!("No country found with name: {}", input_value)
                });
                return HttpResponse::NotFound().json(json_response);
            }
        },
        Err(e) => {
            println!("Error deleting country from database: {}", e);
            let json_response = serde_json::json!({
                "error": "Internal Server Error",
                "details": "Error deleting country from database"
            });
            return HttpResponse::InternalServerError().json(json_response);
        }
    }
}

#[get("/status")]
async fn get_countries_and_last_refreash(_data: web::Data<AppState>) -> impl Responder {
    println!("Received get input for details: countries and last refreash");

    let all_countries = get_all_countries(_data);
    match all_countries {
        Ok(countries) => {
            let last_refreshed = countries[0].last_refreshed_at.clone();
            if last_refreshed.is_none() {
                let json_response = serde_json::json!({
                    "error": "No countries found in database",
                    "details": "Countries data might not have been refreshed yet"
                });
                return HttpResponse::NotFound().json(json_response);
            }
            let json_response = serde_json::json!({
                "total_countries": countries.len(),
                "last_refreshed_at": last_refreshed,
            });
            return HttpResponse::Ok().json(json_response);
        },
        Err(e) => {
            println!("Error retrieving countries from database: {}", e);
            let json_response = serde_json::json!({
                "error": "Internal Server Error",
                "details": "Error retrieving countries from database"
            });
            return HttpResponse::InternalServerError().json(json_response);
        }
    }
}


#[get("/countries/image")]
async fn get_summary_image(req: HttpRequest, _data: web::Data<AppState>) -> impl Responder {
    println!("Received request to generate summary image");
    let path: PathBuf = Path::new("cache/summary.png").to_path_buf();

    if path.exists() {
        match NamedFile::open(&path) {
            Ok(file) => {
                let file = file.set_content_type(mime::IMAGE_PNG);
                // ✅ Pass the real HttpRequest here
                file.into_response(&req)
            }
            Err(_) => HttpResponse::InternalServerError().finish(),
        }
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Summary image not found"
        }))
    }
}


fn generate_summary_image(countries: Vec<Country>, timestamp: String) -> std::io::Result<()> {
    
    // ✅ Prepare directory
    fs::create_dir_all("cache")?;

    // ✅ Calculate stats
    let total_countries = countries.len();

    // sort by estimated GDP descending
    let mut sorted = countries.to_vec();
    sorted.sort_by(|a, b| b.estimated_gdp.partial_cmp(&a.estimated_gdp).unwrap_or(std::cmp::Ordering::Equal));

    let top_5 = sorted.iter().take(5).collect::<Vec<_>>();

    // ✅ Create blank image (800x600 white)
    let mut img = RgbImage::from_pixel(800, 600, Rgb([255, 255, 255]));

    // ✅ Load font (use any TTF file available)
    let font_data = include_bytes!("/System/Library/Fonts/Supplemental/Arial.ttf"); // adjust for your OS
    let font = FontArc::try_from_slice(font_data as &[u8]).unwrap();
    let scale = PxScale { x: 28.0, y: 28.0 };

    // ✅ Draw summary
    draw_text_mut(&mut img, Rgb([0, 0, 0]), 30, 30, scale, &font, 
        &format!("🌍 Country Summary Report"));
    draw_text_mut(&mut img, Rgb([0, 0, 0]), 30, 80, scale, &font,
        &format!("Total Countries: {}", total_countries));
    draw_text_mut(&mut img, Rgb([0, 0, 0]), 30, 120, scale, &font,
        "Top 5 by Estimated GDP:");

    let mut y_offset = 160;
    for (i, c) in top_5.iter().enumerate() {
        draw_text_mut(
            &mut img,
            Rgb([0, 0, 0]),
            50,
            y_offset,
            scale,
            &font,
            &format!("{}. {} - {:.2}", i + 1, c.name.clone().unwrap(), c.estimated_gdp.unwrap_or(0.0)),
        );
        y_offset += 40;
    }

    draw_text_mut(&mut img, Rgb([0, 0, 0]), 30, 400, scale, &font,
        &format!("Last Refreshed: {}", timestamp));

    // ✅ Save image
    match img.save(Path::new("cache/summary.png")){
        Ok(_) =>  println!("✅ Summary image generated at cache/summary.png"),
        Err(e) => {
            println!("Error saving summary image: {}", e);
            return Err(Error::new(std::io::ErrorKind::Other, format!("Error saving summary image: {}", e)));
        }
    }

    Ok(())
}