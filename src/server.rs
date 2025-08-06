use rocket::{get, routes};
use rocket::{Build, Rocket};

use crate::config::Config;

#[get("/hello")]
fn hello() -> &'static str {
    "Hello, Rocket!"
}

pub fn start_http_server(config: &Config) -> Rocket<Build> {
    let config = rocket::Config {
        port: config.http_port,
        ..rocket::Config::default()
    };
    rocket::custom(config).mount("/", routes![hello])
}
