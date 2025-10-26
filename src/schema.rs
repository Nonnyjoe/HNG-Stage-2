// @generated automatically by Diesel CLI.

diesel::table! {
    cache_metadata (id) {
        id -> Integer,
        #[max_length = 255]
        file_path -> Varchar,
        total_countries -> Integer,
        top_countries_json -> Text,
        last_refreshed_at -> Datetime,
    }
}

diesel::table! {
    countries (id) {
        id -> Integer,
        #[max_length = 191]
        name -> Nullable<Varchar>,
        #[max_length = 191]
        capital -> Nullable<Varchar>,
        #[max_length = 191]
        region -> Nullable<Varchar>,
        population -> Nullable<Bigint>,
        #[max_length = 32]
        currency_code -> Nullable<Varchar>,
        exchange_rate -> Nullable<Double>,
        estimated_gdp -> Nullable<Double>,
        #[max_length = 255]
        flag_url -> Nullable<Varchar>,
        last_refreshed_at -> Nullable<Datetime>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(cache_metadata, countries,);
