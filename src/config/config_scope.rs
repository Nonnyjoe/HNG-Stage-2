use crate::routes::{countries_data::{delete_country_data, get_countries_and_last_refreash, get_countries_data, get_country_by_name, get_summary_image, refresh_countries_data}, healthz::check_health};
// use crate::routes::me::me;
use actix_web::web;

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api/v1").service(check_health).service(get_summary_image).service(refresh_countries_data).service(get_countries_data).service(get_country_by_name).service(delete_country_data).service(get_countries_and_last_refreash);
    conf.service(scope);
}
