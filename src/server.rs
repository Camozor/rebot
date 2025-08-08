use std::sync::Arc;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use rocket::{Build, Rocket};
use rocket::{post, routes};

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
        fn is_valid(key: &str) -> bool {
            let my_config = Config::new();
            key == my_config.admin_api_key
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
    my_config: &Config,
    discord_ctx: Arc<RwLock<Option<Arc<serenity::prelude::Context>>>>,
) -> Rocket<Build> {
    let config = rocket::Config {
        port: my_config.http_port,
        ..rocket::Config::default()
    };
    rocket::custom(config)
        .manage(my_config.clone())
        .manage(discord_ctx)
        .mount("/", routes![hello])
}
