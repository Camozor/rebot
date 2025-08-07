use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use rocket::{post, routes};
use rocket::{Build, Rocket};

use crate::config::Config;

struct ApiKey;

#[derive(Debug)]
enum ApiKeyError {
    Missing,
    Invalid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        /// Returns true if `key` is a valid API key string.
        fn is_valid(key: &str) -> bool {
            key == "valid_api_key"
        }

        match req.headers().get_one("authorization") {
            None => Outcome::Error((Status::BadRequest, ApiKeyError::Missing)),
            Some(key) if is_valid(key) => Outcome::Success(ApiKey),
            Some(_) => Outcome::Error((Status::BadRequest, ApiKeyError::Invalid)),
        }
    }
}

#[post("/marius")]
fn hello(_key: ApiKey) -> &'static str {
    "Hello, Rocket!"
}

pub fn start_http_server(config: &Config) -> Rocket<Build> {
    let config = rocket::Config {
        port: config.http_port,
        ..rocket::Config::default()
    };
    rocket::custom(config).mount("/", routes![hello])
}
