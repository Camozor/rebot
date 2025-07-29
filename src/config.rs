use std::env;

use log::debug;

const DEFAULT_CRON_INTERVAL_MINUTE: u32 = 60;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub discord_token: String,
    pub discord_server_id: Option<String>,

    pub database_path: String,

    pub cron_interval_minute: u32,
}

impl Config {
    pub fn new() -> Self {
        if let Err(_) = dotenvy::dotenv() {
            debug!(".env file not found");
        }

        env_logger::init();
        let discord_token = env::var("DISCORD_TOKEN").expect("Configure your discord token bro!");
        let discord_server_id = env::var("DISCORD_SERVER_ID");
        let discord_server_id = match discord_server_id {
            Ok(id) => Some(id),
            Err(_) => None,
        };

        let database_path = env::var("DATABASE_PATH").expect("Configure your database path bro!");

        let cron_interval_minute = env::var("CRON_INTERVAL_MINUTE");
        let cron_interval_minute = match cron_interval_minute {
            Ok(interval) => {
                let number = interval.trim().parse();
                match number {
                    Ok(n) => n,
                    Err(_) => DEFAULT_CRON_INTERVAL_MINUTE,
                }
            }
            Err(_) => DEFAULT_CRON_INTERVAL_MINUTE,
        };

        Config {
            discord_token,
            discord_server_id,
            database_path,
            cron_interval_minute,
        }
    }
}
