use std::env;

use log::debug;

const DEFAULT_CRON_INTERVAL_MINUTE: u32 = 60;
const DEFAULT_HTTP_PORT: u16 = 8000;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub discord_token: String,
    pub discord_server_id: Option<String>,

    pub http_port: u16,

    pub database_path: String,

    pub cron_interval_minute: u32,
    pub skip_cron: bool,
}

impl Config {
    pub fn new() -> Self {
        if let Err(_) = dotenvy::dotenv() {
            debug!(".env file not found");
        }

        env_logger::init();
        let discord_token = env::var("DISCORD_TOKEN").expect("Configure your discord token bro!");
        let discord_server_id = env::var("DISCORD_SERVER_ID").ok();

        let http_port = env::var("HTTP_PORT")
            .ok()
            .and_then(|port| port.trim().parse().ok())
            .unwrap_or(DEFAULT_HTTP_PORT);

        let database_path = env::var("DATABASE_PATH").expect("Configure your database path bro!");

        let cron_interval_minute = env::var("CRON_INTERVAL_MINUTE")
            .ok()
            .and_then(|interval| interval.trim().parse().ok())
            .unwrap_or(DEFAULT_CRON_INTERVAL_MINUTE);

        let skip_cron: bool = env::var("SKIP_CRON")
            .unwrap_or(String::from("false"))
            .parse()
            .unwrap_or(false);

        Config {
            discord_token,
            discord_server_id,
            http_port,
            database_path,
            cron_interval_minute,
            skip_cron,
        }
    }
}
