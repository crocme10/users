use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE};
use std::env;

// use super::server::run_server;
// // use users::api::model::User;
// use users::api::users::{MultiUsersResponseBody, SingleUserResponseBody};
// use users::db::pg;
// use users::error;
// use users::settings::Settings;

pub fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

pub fn get_service_url() -> String {
    let mode = env::var("RUN_MODE").expect("RUN_MODE should be set");
    match mode.as_str() {
        "testing" => String::from("http://localhost:8081/graphql"),
        _ => String::from("http://users:8081/graphql"),
    }
}

pub fn get_database_url() -> String {
    let mode = env::var("RUN_MODE").expect("RUN_MODE should be set");
    match mode.as_str() {
        "testing" => env::var("DATABASE_TEST_URL").expect("DATABASE_TEST_URL should be set"),
        _ => env::var("DATABASE_URL").expect("DATABASE_URL should be set"),
    }
}
