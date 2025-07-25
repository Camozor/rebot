use std::{
    fs::{self, File},
    io::Write,
};

use log::{debug, error};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    model::player_stat::Rank,
    scraper::{Scraper, ScraperInitError},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisteredPlayer {
    pub discord_id: u64,
    pub rematch_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerWithStats {
    pub discord_id: u64,
    pub display_name: String,
    pub rank: Rank,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerStore {
    #[serde(skip)]
    pub config: Config,

    pub registered_players: Vec<RegisteredPlayer>,
    pub players: Vec<PlayerWithStats>,
}

#[derive(Debug)]
pub enum RegisterError {
    WrongUrl(String),
}

#[derive(Debug)]
pub enum RefreshError {
    Err,
}

impl PlayerStore {
    pub fn new(config: Config) -> Self {
        PlayerStore {
            config: config,
            registered_players: vec![],
            players: vec![],
        }
    }

    pub fn load_database(config: Config) -> Self {
        let json_data = fs::read_to_string(&config.database_path);
        let json_data = match json_data {
            Ok(v) => v,
            Err(e) => {
                error!("Could not read {} database, {}", config.database_path, e);
                return PlayerStore::new(config);
            }
        };

        let store: PlayerStore = serde_json::from_str(&json_data).unwrap_or_else(|e| {
            error!("Could not parse {} database, {}", config.database_path, e);
            return PlayerStore::new(config.clone());
        });

        PlayerStore {
            config: config,
            registered_players: store.registered_players,
            players: store.players,
        }
    }

    pub fn print(&self) {
        debug!("Registered players: {:?}", self.registered_players);
        debug!("Players: {:?}", self.players);
    }

    pub fn write_database(&self) {
        let json_data = serde_json::to_string_pretty(self);
        let json_data = match json_data {
            Ok(json_str) => json_str,
            Err(e) => {
                error!("Could not stringify store, {}", e);
                return;
            }
        };

        let file = File::create(&self.config.database_path);
        let mut file = match file {
            Ok(f) => f,
            Err(e) => {
                error!("Could not create {} file, {}", self.config.database_path, e);
                return;
            }
        };

        let written_bytes = file.write(json_data.as_bytes());
        if let Err(e) = written_bytes {
            error!("Could write to {} file, {}", self.config.database_path, e);
        }
    }

    pub fn register_player(
        &mut self,
        discord_id: u64,
        rematch_url: &str,
    ) -> Result<(), RegisterError> {
        if !rematch_url.starts_with("https://u.gg/rematch/profile/") {
            return Err(RegisterError::WrongUrl(String::from(
                "ton URL doit ressembler à https://u.gg/rematch/profile/steam/La%20m%C3%A9sange%20du%20Val%20d'Oise/76561198355389674",
            )));
        }

        let existing_player = self
            .registered_players
            .iter_mut()
            .find(|p| p.discord_id == discord_id);

        match existing_player {
            Some(player) => player.rematch_url = rematch_url.to_owned(),
            None => self.registered_players.push(RegisteredPlayer {
                discord_id,
                rematch_url: rematch_url.to_owned(),
            }),
        }

        Ok(())
    }

    pub async fn refresh_all(&mut self) -> Result<(), RefreshError> {
        let scraper = Scraper::new().await;
        let mut scraper = match scraper {
            Ok(scraper) => scraper,
            Err(ScraperInitError::Browser(error)) => {
                error!("Error initializing scraper: {}", error);
                return Err(RefreshError::Err);
            }
        };

        debug!("Scraper created");

        let result = scraper.get_players_stats(&self.registered_players).await;
        self.players = result;

        self.write_database();

        Ok(())
    }

    pub fn get_player_stat(&self, discord_id: u64) -> Option<&PlayerWithStats> {
        self.players
            .iter()
            .find(|player| player.discord_id == discord_id)
    }
}
