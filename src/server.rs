use std::sync::Arc;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use rocket::{post, routes};
use rocket::{Build, Rocket};

use crate::config::Config;
use crate::discord::play_marius;
use tokio::sync::RwLock;

use rocket::State;

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
async fn hello(
    _key: ApiKey,
    discord_ctx: &State<Arc<RwLock<Option<Arc<serenity::prelude::Context>>>>>,
) -> &'static str {
    let ctx_lock = discord_ctx.read().await;

    match &*ctx_lock {
        Some(ctx) => {
            // TODO parse body
            let _ = play_marius(ctx, 997501722589143163.into(), 428258972156559362.into()).await;
            "Success"
        }
        None => "Fail",
    }
}

pub fn start_http_server(
    config: &Config,
    discord_ctx: Arc<RwLock<Option<Arc<serenity::prelude::Context>>>>,
) -> Rocket<Build> {
    let config = rocket::Config {
        port: config.http_port,
        ..rocket::Config::default()
    };
    rocket::custom(config)
        .manage(discord_ctx)
        .mount("/", routes![hello])
}
