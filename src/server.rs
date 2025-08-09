use std::sync::Arc;

use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use rocket::serde::{Deserialize, json::Json};
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

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct MariusBody {
    guild_id: u64,
    user_id: u64,
}

#[post("/marius", data = "<body>")]
async fn marius(
    _key: ApiKey,
    discord_ctx: &State<Arc<RwLock<Option<Arc<serenity::prelude::Context>>>>>,
    body: Json<MariusBody>,
) -> &'static str {
    let ctx_lock = discord_ctx.read().await;

    match &*ctx_lock {
        Some(ctx) => {
            let _ = play_marius(ctx, body.guild_id.into(), body.user_id.into()).await;
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
        address: "0.0.0.0".parse().unwrap(),
        ..rocket::Config::default()
    };
    rocket::custom(config)
        .manage(my_config.clone())
        .manage(discord_ctx)
        .mount("/", routes![marius])
}
